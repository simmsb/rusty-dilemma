use core::hash::Hash;
use core::mem::MaybeUninit;
use defmt::{debug, warn, Format};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, mutex::Mutex, pubsub::Publisher,
};
use embassy_time::{with_timeout, Duration};
use futures::Future;
use heapless::Arc;
use postcard::accumulator::{CobsAccumulator, FeedResult};
use serde::{de::DeserializeOwned, Serialize};
use shared::cmd::{CmdOrAck, Command};

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
    U: DeserializeOwned + Hash + Format + Clone,
    RX: embedded_io::asynch::Read,
{
    async fn recv_task_inner(&mut self) -> Option<()> {
        let mut accumulator = CobsAccumulator::<BUF_SIZE>::new();

        loop {
            let mut buf = [0u8; 1];
            self.rx.read(&mut buf).await.ok()?;
            let mut window = &buf[..];

            'cobs: while !window.is_empty() {
                window = match accumulator.feed(window) {
                    FeedResult::Consumed => break 'cobs,
                    FeedResult::OverFull(buf) => buf,
                    FeedResult::DeserError(buf) => {
                        warn!(
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
                                    debug!("Received command: {:?}", c);
                                    if let Some(ack) = c.ack() {
                                        self.mix_chan.send(CmdOrAck::Ack(ack)).await;
                                        self.out_chan.publish(c.cmd).await;
                                    }
                                } else {
                                    warn!("Corrupted parsed command: {:?}", c);
                                }
                            }
                            CmdOrAck::Ack(a) => {
                                if let Some(a) = a.validate() {
                                    debug!("Received ack: {:?}", a);
                                    let mut waiters = self.waiters.lock().await;
                                    if let Some(waker) = waiters.remove(&a.uuid) {
                                        waker.set();
                                    }
                                } else {
                                    warn!("Corrupted parsed ack");
                                }
                            }
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
    T: Serialize + Format,
    TX: embedded_io::asynch::Write,
    <TX as embedded_io::Io>::Error: Format,
{
    async fn task(self) {
        loop {
            let val = self.mix_chan.recv().await;

            let mut buf = [0u8; BUF_SIZE];
            if let Ok(buf) = postcard::to_slice_cobs(&val, &mut buf) {
                let r = self.tx.write(buf).await;
                debug!("Transmitted {:?}, r: {:?}", val, r);
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
            let (cmd, uuid) = Command::new_reliable(cmd.clone());
            let waiter = self.register_waiter(uuid).await;
            self.mix_chan.send(CmdOrAck::Cmd(cmd)).await;

            match with_timeout(timeout, waiter.wait()).await {
                Ok(_) => {
                    debug!("Waiter for uuid {} completed", uuid);
                    return;
                }
                Err(_) => {
                    warn!("Waiter for uuid{} timing out", uuid);
                    self.deregister_waiter(uuid).await;
                }
            }
        }
    }

    async fn register_waiter(&self, uuid: u8) -> Arc<P> {
        let signal = P::alloc(Event::new()).unwrap();
        let mut waiters = self.waiters.lock().await;
        if waiters.insert(uuid, signal.clone()).is_ok() {
            signal
        } else {
            panic!("Duped waiter uuid")
        }
    }

    async fn deregister_waiter(&self, uuid: u8) {
        self.waiters.lock().await.remove(&uuid);
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
        T: Hash + Clone + Serialize + Format,
        U: Hash + Clone + DeserializeOwned + Format,
        TX: embedded_io::asynch::Write,
        RX: embedded_io::asynch::Read,
        <TX as embedded_io::Io>::Error: Format,
    {
        let sender = EventSender {
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
