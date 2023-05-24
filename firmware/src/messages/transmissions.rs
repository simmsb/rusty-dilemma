use core::hash::Hash;
use embassy_futures::select;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{with_timeout, Duration};
use futures::Future;
use postcard::accumulator::{CobsAccumulator, FeedResult};
use serde::{de::DeserializeOwned, Serialize};
use shared::cmd::{CmdOrAck, Command};

use crate::utils::{log, WhichDebug};

use super::TransmittedMessage;

const BUF_SIZE: usize = 128;

struct EventSenderImpl<'e, T> {
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<T>, 16>,
    ack_chan: &'e Channel<ThreadModeRawMutex, (), 4>,
}

pub trait EventSender<T> {
    async fn send(&self, cmd: TransmittedMessage<T>) {
        let TransmittedMessage { msg, timeout } = cmd;
        if let Some(timeout) = timeout {
            let _ = self.send_reliable(msg, timeout).await;
        } else {
            let _ = self.send_unreliable(msg).await;
        }
    }

    async fn send_unreliable(&self, cmd: T);
    async fn send_reliable(&self, cmd: T, timeout: Duration);
}

struct EventOutProcessor<'e, Sent, TX> {
    tx: TX,
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<Sent>, 16>,
}

struct EventInProcessor<'e, Sent, RX, FnTx> {
    rx: RX,
    out_cb: FnTx,
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<Sent>, 16>,
    ack_chan: &'e Channel<ThreadModeRawMutex, (), 4>,
}

impl<'e, Sent, RX, FnTx> EventInProcessor<'e, Sent, RX, FnTx>
where
    RX: embedded_io::asynch::Read,
{
    async fn recv_task_inner<Received, FnTxFut>(
        &mut self,
    ) -> Result<(), <RX as embedded_io::Io>::Error>
    where
        Received: DeserializeOwned + Hash + Clone + WhichDebug,
        FnTxFut: Future,
        FnTx: Fn(Received) -> FnTxFut,
    {
        let mut accumulator = CobsAccumulator::<BUF_SIZE>::new();

        loop {
            let mut buf = [0u8; BUF_SIZE];
            let n = self.rx.read(&mut buf).await?;
            let mut window = &buf[..n];

            // log::info!("cobs: {}", accumulator);

            'cobs: while !window.is_empty() {
                window = match accumulator.feed(window) {
                    FeedResult::Consumed => break 'cobs,
                    FeedResult::OverFull(buf) => {
                        log::debug!("buffer overfull");
                        buf
                    }
                    FeedResult::DeserError(buf) => {
                        log::debug!(
                            "Message decoder failed to deserialize a message of type {}: {:?}",
                            core::any::type_name::<CmdOrAck<Received>>(),
                            buf
                        );
                        buf
                    }
                    FeedResult::Success { data, remaining } => {
                        let data: CmdOrAck<Received> = data;

                        match data {
                            CmdOrAck::Cmd(c) => {
                                if c.validate() {
                                    // log::info!("Hi I got a command: {}", c);
                                    let send_ack = c.reliable;
                                    (self.out_cb)(c.cmd).await;
                                    if send_ack {
                                        self.mix_chan.send(CmdOrAck::Ack).await;
                                    }
                                } else {
                                    // log::debug!("Corrupted parsed command: {:?}", c);
                                }
                            }
                            CmdOrAck::Ack => {
                                self.ack_chan.send(()).await;
                            }
                        }

                        remaining
                    }
                }
            }
        }
    }

    async fn task<Received, FnTxFut>(&mut self)
    where
        Received: DeserializeOwned + Hash + Clone + WhichDebug,
        FnTxFut: Future,
        FnTx: Fn(Received) -> FnTxFut,
        <RX as embedded_io::Io>::Error: WhichDebug,
    {
        loop {
            let _r = self.recv_task_inner().await;
            // log::debug!("Restarting cobs receiver for reason: {}", _r);
        }
    }
}

impl<'e, T, TX> EventOutProcessor<'e, T, TX>
where
    T: Serialize + WhichDebug,
    TX: embedded_io::asynch::Write,
    <TX as embedded_io::Io>::Error: WhichDebug,
{
    async fn task(&mut self) {
        loop {
            let val = self.mix_chan.recv().await;

            let mut buf = [0u8; BUF_SIZE];
            if let Ok(buf) = postcard::to_slice_cobs(&val, &mut buf) {
                let _r = self.tx.write_all(buf).await;
                // log::debug!("Transmitted {:?} as {:?}, r: {:?}", val, buf, _r);
            }
        }
    }
}

impl<'a, T: Hash + Clone> EventSender<T> for EventSenderImpl<'a, T> {
    async fn send_unreliable(&self, cmd: T) {
        let cmd = Command::new_unreliable(cmd.clone());
        self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;
    }

    async fn send_reliable(&self, cmd: T, mut timeout: Duration) {
        loop {
            let cmd = Command::new_reliable(cmd.clone());
            self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;

            match with_timeout(timeout, self.ack_chan.recv()).await {
                Ok(_) => {
                    // log::debug!("Waiter for id {} completed", id);
                    return;
                }
                Err(_) => {
                    // log::debug!("Waiter for id {} timing out", id);
                }
            }

            timeout += Duration::from_micros(100);
        }
    }
}

pub async fn eventer<Sent, Received, TX, RX, FnRx, FnTx, FnRxFut, FnTxFut>(
    tx: TX,
    rx: RX,
    fn_rx: FnRx,
    fn_tx: FnTx,
) where
    Sent: Hash + Clone + Serialize + WhichDebug,
    Received: Hash + Clone + DeserializeOwned + WhichDebug,
    TX: embedded_io::asynch::Write,
    RX: embedded_io::asynch::Read,
    <RX as embedded_io::Io>::Error: WhichDebug,
    <TX as embedded_io::Io>::Error: WhichDebug,
    FnRx: Fn() -> FnRxFut,
    FnTx: Fn(Received) -> FnTxFut,
    FnRxFut: Future<Output = TransmittedMessage<Sent>>,
    FnTxFut: Future,
{
    let mix_chan = Channel::new();
    let ack_chan = Channel::new();

    let sender = EventSenderImpl {
        mix_chan: &mix_chan,
        ack_chan: &ack_chan,
    };

    let mut out_processor = EventOutProcessor::<Sent, TX> {
        tx,
        mix_chan: &mix_chan,
    };

    let mut in_processor = EventInProcessor::<Sent, RX, FnTx> {
        rx,
        out_cb: fn_tx,
        mix_chan: &mix_chan,
        ack_chan: &ack_chan,
    };

    let sender_proc = async {
        loop {
            let TransmittedMessage { msg, timeout } = fn_rx().await;
            if let Some(timeout) = timeout {
                let _ = sender.send_reliable(msg, timeout).await;
            } else {
                let _ = sender.send_unreliable(msg).await;
            }
        }
    };

    select::select3(sender_proc, out_processor.task(), in_processor.task()).await;
}
