use core::hash::Hash;
use core::mem::MaybeUninit;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, mutex::Mutex, pubsub::Publisher,
};
use embassy_time::{with_timeout, Duration};
use futures::Future;
use heapless::Arc;
use postcard::accumulator::{CobsAccumulator, FeedResult};
use serde::{de::DeserializeOwned, Serialize};
use shared::cmd::{CmdOrAck, Command};

#[cfg(feature = "probe")]
use defmt as log;

#[cfg(not(feature = "probe"))]
pub trait WhichDebug = ::core::fmt::Debug;
#[cfg(feature = "probe")]
pub trait WhichDebug = ::defmt::Format;

use crate::{event::Event, utils};

use super::TransmittedMessage;

const BUF_SIZE: usize = 128;

heapless::arc_pool!(P: Event);

pub fn init_pool() {
    let memory: &'static mut [u8; 8192] = utils::singleton!([0u8; 8192]);

    P::grow(&mut memory[..]);
}

pub struct Eventer<T, TX, RX> {
    tx: TX,
    rx: RX,
    mix_chan: Channel<ThreadModeRawMutex, CmdOrAck<T>, 16>,
    waiters: Mutex<ThreadModeRawMutex, heapless::FnvIndexMap<u8, Arc<P>, 128>>,
}

struct EventSender<'e, T> {
    id_gen: atomic_polyfill::AtomicU8,
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<T>, 16>,
    waiters: &'e Mutex<ThreadModeRawMutex, heapless::FnvIndexMap<u8, Arc<P>, 128>>,
}

struct EventOutProcessor<'e, T, TX> {
    tx: &'e mut TX,
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<T>, 16>,
}

struct EventInProcessor<
    'a,
    'e,
    T,
    U: Clone,
    RX,
    const CAP: usize,
    const SUBS: usize,
    const PUBS: usize,
> {
    rx: &'e mut RX,
    out_chan: Publisher<'a, ThreadModeRawMutex, U, CAP, SUBS, PUBS>,
    mix_chan: &'e Channel<ThreadModeRawMutex, CmdOrAck<T>, 16>,
    waiters: &'e Mutex<ThreadModeRawMutex, heapless::FnvIndexMap<u8, Arc<P>, 128>>,
}

impl<'a, 'e, T, U, RX, const CAP: usize, const SUBS: usize, const PUBS: usize>
    EventInProcessor<'a, 'e, T, U, RX, CAP, SUBS, PUBS>
where
    U: DeserializeOwned + Hash + Clone + WhichDebug,
    RX: embedded_io::asynch::Read,
{
    async fn recv_task_inner(&mut self) -> Option<()> {
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
                        log::warn!(
                            "Message decoder failed to deserialize a message of type {}: {:?}",
                            core::any::type_name::<CmdOrAck<U>>(),
                            buf
                        );
                        buf
                    }
                    FeedResult::Success { data, remaining } => {
                        let data: CmdOrAck<U> = data;

                        match data {
                            CmdOrAck::Cmd(c) => {
                                if c.validate() {
                                    log::debug!("Received command: {:?}", c);
                                    log::info!("Got a command");
                                    if let Some(ack) = c.ack() {
                                        self.mix_chan.send(CmdOrAck::Ack(ack)).await;
                                    }
                                    self.out_chan.publish(c.cmd).await;
                                } else {
                                    log::warn!("Corrupted parsed command: {:?}", c);
                                }
                            }
                            CmdOrAck::Ack(a) => match a.validate() {
                                Ok(a) => {
                                    log::debug!("Received ack: {:?}", a);
                                    let mut waiters = self.waiters.lock().await;
                                    if let Some(waker) = waiters.remove(&a.id) {
                                        waker.set();
                                    }
                                }
                                Err(e) => {
                                    log::warn!("Corrupted parsed ack: {:?}", e);
                                }
                            },
                        }

                        remaining
                    }
                }
            }
        }
    }

    async fn task(mut self) {
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
    async fn task(self) {
        loop {
            let val = self.mix_chan.recv().await;

            let mut buf = [0u8; BUF_SIZE];
            if let Ok(buf) = postcard::to_slice_cobs(&val, &mut buf) {
                let r = self.tx.write(buf).await;
                log::debug!("Transmitted {:?}, r: {:?}", val, r);
            }
        }
    }
}

impl<'a, T: Hash + Clone> EventSender<'a, T> {
    async fn send_unreliable(&self, cmd: T) {
        let cmd = Command::new_unreliable(cmd.clone());
        self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;
    }

    async fn send_reliable(&self, cmd: T, timeout: Duration) {
        loop {
            let id = self
                .id_gen
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            let cmd = Command::new_reliable(cmd.clone(), id);
            let waiter = self.register_waiter(id).await;
            self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;

            match with_timeout(timeout, waiter.wait()).await {
                Ok(_) => {
                    log::debug!("Waiter for id {} completed", id);
                    return;
                }
                Err(_) => {
                    log::warn!("Waiter for id{} timing out", id);
                    self.deregister_waiter(id).await;
                }
            }
        }
    }

    async fn register_waiter(&self, id: u8) -> Arc<P> {
        let signal = P::alloc(Event::new()).unwrap();
        let mut waiters = self.waiters.lock().await;
        if waiters.insert(id, signal.clone()).is_ok() {
            signal
        } else {
            panic!("Duped waiter id")
        }
    }

    async fn deregister_waiter(&self, id: u8) {
        self.waiters.lock().await.remove(&id);
    }
}

impl<'a, T, TX, RX> Eventer<T, TX, RX> {
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            tx,
            rx,
            mix_chan: Channel::new(),
            waiters: Mutex::new(heapless::FnvIndexMap::new()),
        }
    }

    pub fn split_tasks<
        's,
        U,
        const N: usize,
        const CAP: usize,
        const SUBS: usize,
        const PUBS: usize,
    >(
        &'s mut self,
        cmd_chan: &'static Channel<ThreadModeRawMutex, TransmittedMessage<T>, N>,
        out_chan: Publisher<'static, ThreadModeRawMutex, U, CAP, SUBS, PUBS>,
    ) -> (impl Future + 's, impl Future + 's, impl Future + 's)
    where
        T: Hash + Clone + Serialize + WhichDebug,
        U: Hash + Clone + DeserializeOwned + WhichDebug,
        TX: embedded_io::asynch::Write,
        RX: embedded_io::asynch::Read,
        <TX as embedded_io::Io>::Error: WhichDebug,
    {
        let sender = EventSender {
            id_gen: atomic_polyfill::AtomicU8::new(0),
            mix_chan: &self.mix_chan,
            waiters: &self.waiters,
        };

        let out_processor = EventOutProcessor {
            tx: &mut self.tx,
            mix_chan: &self.mix_chan,
        };

        let in_processor = EventInProcessor {
            rx: &mut self.rx,
            out_chan,
            mix_chan: &self.mix_chan,
            waiters: &self.waiters,
        };

        let sender_proc = async move {
            loop {
                let TransmittedMessage { msg, timeout } = cmd_chan.recv().await;
                if let Some(timeout) = timeout {
                    let _ = sender.send_reliable(msg, timeout).await;
                } else {
                    let _ = sender.send_unreliable(msg).await;
                }
            }
        };

        (sender_proc, out_processor.task(), in_processor.task())
    }
}
