use core::hash::Hash;
use embassy_futures::select;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, signal::Signal};
use embassy_time::{with_timeout, Duration};
use futures::Future;
use postcard::accumulator::{CobsAccumulator, FeedResult};
use serde::{de::DeserializeOwned, Serialize};
use shared::cmd::{CmdOrAck, Command};

use crate::utils::WhichDebug;

use super::TransmittedMessage;

const BUF_SIZE: usize = 128;

struct EventSenderImpl<'e, T> {
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<T>, 16>,
    ack_signal: &'e Signal<ThreadModeRawMutex, ()>,
}

pub trait EventSender<T> {
    async fn send(&self, cmd: TransmittedMessage<T>, id: u8) {
        let TransmittedMessage { msg, timeout } = cmd;
        if let Some(timeout) = timeout {
            let _ = self.send_reliable(msg, timeout, id).await;
        } else {
            let _ = self.send_unreliable(msg, id).await;
        }
    }

    async fn send_unreliable(&self, cmd: T, id: u8);
    async fn send_reliable(&self, cmd: T, timeout: Duration, id: u8);
}

struct EventOutProcessor<'e, Sent, TX> {
    tx: TX,
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<Sent>, 16>,
}

struct EventInProcessor<'e, Sent, RX, FnTx> {
    rx: RX,
    out_cb: FnTx,
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<Sent>, 16>,
    ack_signal: &'e Signal<ThreadModeRawMutex, ()>,
}

impl<'e, Sent, RX, FnTx> EventInProcessor<'e, Sent, RX, FnTx>
where
    RX: embedded_io_async::Read,
{
    async fn receive_task_inner<Received, FnTxFut>(
        &mut self,
    ) -> Result<(), <RX as embedded_io_async::ErrorType>::Error>
    where
        Received: DeserializeOwned + Hash + Clone + WhichDebug,
        FnTxFut: Future,
        FnTx: Fn(Received) -> FnTxFut,
    {
        let mut accumulator = CobsAccumulator::<BUF_SIZE>::new();
        let mut last_seen_id = None;

        loop {
            let mut buf = [0u8; BUF_SIZE];
            let n = self.rx.read(&mut buf).await?;
            let mut window = &buf[..n];

            // log::info!("cobs: {}", accumulator);

            'cobs: while !window.is_empty() {
                window = match accumulator.feed(window) {
                    FeedResult::Consumed => break 'cobs,
                    FeedResult::OverFull(buf) => {
                        // log::debug!("buffer overfull");
                        buf
                    }
                    FeedResult::DeserError(buf) => {
                        // log::debug!(
                        //     "Message decoder failed to deserialize a message of type {}: {:?}",
                        //     core::any::type_name::<CmdOrAck<Received>>(),
                        //     buf
                        // );
                        buf
                    }
                    FeedResult::Success { data, remaining } => {
                        let data: CmdOrAck<Received> = data;

                        match data {
                            CmdOrAck::Cmd(c) => {
                                if c.validate() {
                                    // log::info!("Hi I got a command: {}", c);
                                    if c.command_seq.reliable() {
                                        self.mix_chan.send(CmdOrAck::Ack).await;
                                    }
                                    if Some(c.command_seq.id()) != last_seen_id {
                                        (self.out_cb)(c.cmd).await;
                                        last_seen_id = Some(c.command_seq.id());
                                    }
                                } else {
                                    // log::debug!("Corrupted parsed command: {:?}", c);
                                }
                            }
                            CmdOrAck::Ack => {
                                self.ack_signal.signal(());
                            }
                        }

                        remaining
                    }
                };
            }
        }
    }

    async fn task<Received, FnTxFut>(&mut self)
    where
        Received: DeserializeOwned + Hash + Clone + WhichDebug,
        FnTxFut: Future,
        FnTx: Fn(Received) -> FnTxFut,
        <RX as embedded_io_async::ErrorType>::Error: WhichDebug,
    {
        loop {
            let _r = self.receive_task_inner().await;
            // log::debug!("Restarting cobs receiver for reason: {}", _r);
        }
    }
}

impl<'e, T, TX> EventOutProcessor<'e, T, TX>
where
    T: Serialize + WhichDebug,
    TX: embedded_io_async::Write,
    <TX as embedded_io_async::ErrorType>::Error: WhichDebug,
{
    async fn task(&mut self) {
        loop {
            let val = self.mix_chan.receive().await;

            let mut buf = [0u8; BUF_SIZE];
            if let Ok(buf) = postcard::to_slice_cobs(&val, &mut buf) {
                let _r = self.tx.write_all(buf).await;
                // log::debug!("Transmitted {:?} as {:?}, r: {:?}", val, buf, _r);
            }
        }
    }
}

impl<'a, T: Hash + Clone> EventSender<T> for EventSenderImpl<'a, T> {
    async fn send_unreliable(&self, cmd: T, id: u8) {
        let cmd = Command::new_unreliable(cmd.clone(), id);
        self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;
    }

    async fn send_reliable(&self, cmd: T, mut timeout: Duration, id: u8) {
        loop {
            let cmd = Command::new_reliable(cmd.clone(), id);
            self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;

            self.ack_signal.reset();

            if with_timeout(timeout, self.ack_signal.wait()).await.is_ok() {
                // log::debug!("Waiter for id {} completed", id);
                return;
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
    TX: embedded_io_async::Write,
    RX: embedded_io_async::Read,
    <RX as embedded_io_async::ErrorType>::Error: WhichDebug,
    <TX as embedded_io_async::ErrorType>::Error: WhichDebug,
    FnRx: Fn() -> FnRxFut,
    FnTx: Fn(Received) -> FnTxFut,
    FnRxFut: Future<Output = TransmittedMessage<Sent>>,
    FnTxFut: Future,
{
    let mix_chan = Channel::new();
    let ack_signal = Signal::new();

    let sender = EventSenderImpl {
        mix_chan: &mix_chan,
        ack_signal: &ack_signal,
    };

    let mut out_processor = EventOutProcessor::<Sent, TX> {
        tx,
        mix_chan: &mix_chan,
    };

    let mut in_processor = EventInProcessor::<Sent, RX, FnTx> {
        rx,
        out_cb: fn_tx,
        mix_chan: &mix_chan,
        ack_signal: &ack_signal,
    };

    let sender_proc = async {
        let mut id: u8 = 0;
        loop {
            let TransmittedMessage { msg, timeout } = fn_rx().await;
            if let Some(timeout) = timeout {
                let _ = sender.send_reliable(msg, timeout, id).await;
            } else {
                let _ = sender.send_unreliable(msg, id).await;
            }
            id += 1;
            id &= 0b1111111;
        }
    };

    select::select3(sender_proc, out_processor.task(), in_processor.task()).await;
}
