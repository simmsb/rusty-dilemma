use core::hash::Hash;
use embassy_futures::select;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{with_timeout, Duration};
use futures::Future;
use postcard::accumulator::{CobsAccumulator, FeedResult};
use serde::{de::DeserializeOwned, Serialize};
use shared::cmd::{CmdOrAck, Command};

#[cfg(feature = "probe")]
use defmt as log;

#[cfg(not(feature = "probe"))]
pub trait WhichDebug = ::core::fmt::Debug;
#[cfg(feature = "probe")]
pub trait WhichDebug = ::defmt::Format;

use crate::event::Event;

use super::TransmittedMessage;

const BUF_SIZE: usize = 128;

struct EventSenderImpl<'e, T> {
    id_chan: &'e Channel<ThreadModeRawMutex, u8, 128>,
    events: &'e [Event],
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<T>, 16>,
    waiters: &'e Mutex<ThreadModeRawMutex, heapless::FnvIndexSet<u8, 128>>,
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
    events: &'e [Event],
    waiters: &'e Mutex<ThreadModeRawMutex, heapless::FnvIndexSet<u8, 128>>,
}

impl<'e, Sent, RX, FnTx> EventInProcessor<'e, Sent, RX, FnTx>
where
    RX: embedded_io::asynch::Read,
{
    async fn recv_task_inner<Received, FnTxFut>(&mut self) -> Option<()>
    where
        Received: DeserializeOwned + Hash + Clone + WhichDebug,
        FnTxFut: Future,
        FnTx: Fn(Received) -> FnTxFut,
    {
        let mut accumulator = CobsAccumulator::<BUF_SIZE>::new();

        loop {
            let mut buf = [0u8; 16];
            let n = self.rx.read(&mut buf).await.ok()?;
            let mut window = &buf[..n];

            'cobs: while !window.is_empty() {
                window = match accumulator.feed(window) {
                    FeedResult::Consumed => break 'cobs,
                    FeedResult::OverFull(buf) => buf,
                    FeedResult::DeserError(buf) => {
                        // log::debug!(
                        //     "Message decoder failed to deserialize a message of type {}: {:?}",
                        //     core::any::type_name::<CmdOrAck<U>>(),
                        //     buf
                        // );
                        buf
                    }
                    FeedResult::Success { data, remaining } => {
                        let data: CmdOrAck<Received> = data;

                        match data {
                            CmdOrAck::Cmd(c) => {
                                if c.validate() {
                                    if let Some(ack) = c.ack() {
                                        self.mix_chan.send(CmdOrAck::Ack(ack)).await;
                                    }
                                    (self.out_cb)(c.cmd).await;
                                } else {
                                    // log::debug!("Corrupted parsed command: {:?}", c);
                                }
                            }
                            CmdOrAck::Ack(a) => match a.validate() {
                                Ok(a) => {
                                    // log::debug!("Received ack: {:?}", a);
                                    let mut waiters = self.waiters.lock().await;
                                    if waiters.remove(&a.id) {
                                        self.events[a.id as usize].set();
                                    }
                                }
                                Err(e) => {
                                    // log::debug!("Corrupted parsed ack: {:?}", e);
                                }
                            },
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
    {
        loop {
            let _ = self.recv_task_inner().await;
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
                let _r = self.tx.write(buf).await;
                // log::debug!("Transmitted {:?}, r: {:?}", val, r);
            }
        }
    }
}

impl<'a, T: Hash + Clone> EventSenderImpl<'a, T> {
    async fn register_waiter(&self, id: u8, event: &'a Event) {
        let mut waiters = self.waiters.lock().await;
        if waiters.insert(id).is_err() {
            panic!("Duped waiter id")
        }
    }

    async fn deregister_waiter(&self, id: u8) {
        self.waiters.lock().await.remove(&id);
    }
}

impl<'a, T: Hash + Clone> EventSender<T> for EventSenderImpl<'a, T> {
    async fn send_unreliable(&self, cmd: T) {
        let cmd = Command::new_unreliable(cmd.clone());
        self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;
    }

    async fn send_reliable(&self, cmd: T, timeout: Duration) {
        loop {
            let id = self.id_chan.recv().await;
            let waiter = &self.events[id as usize];
            waiter.reset();
            let cmd = Command::new_reliable(cmd.clone(), id);
            self.register_waiter(id, waiter).await;
            self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;

            match with_timeout(timeout, waiter.wait()).await {
                Ok(_) => {
                    log::debug!("Waiter for id {} completed", id);
                    self.id_chan.send(id).await;
                    return;
                }
                Err(_) => {
                    log::debug!("Waiter for id {} timing out", id);
                    self.deregister_waiter(id).await;
                    self.id_chan.send(id).await;
                }
            }
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
    <TX as embedded_io::Io>::Error: WhichDebug,
    FnRx: Fn() -> FnRxFut,
    FnTx: Fn(Received) -> FnTxFut,
    FnRxFut: Future<Output = TransmittedMessage<Sent>>,
    FnTxFut: Future,
{
    const E: Event = Event::new();
    let events = [E; 128];
    let id_chan = Channel::<ThreadModeRawMutex, u8, 128>::new();
    let mix_chan = Channel::new();
    let waiters = Mutex::new(heapless::FnvIndexSet::new());

    for (i, _) in events.iter().enumerate() {
        id_chan.try_send(i as u8).unwrap();
    }

    let sender = EventSenderImpl {
        id_chan: &id_chan,
        events: &events,
        mix_chan: &mix_chan,
        waiters: &waiters,
    };

    let mut out_processor = EventOutProcessor::<Sent, TX> {
        tx,
        mix_chan: &mix_chan,
    };

    let mut in_processor = EventInProcessor::<Sent, RX, FnTx> {
        rx,
        out_cb: fn_tx,
        mix_chan: &mix_chan,
        events: &events,
        waiters: &waiters,
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
