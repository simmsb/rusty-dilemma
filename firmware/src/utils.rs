use core::{arch::asm, marker::PhantomData};

#[cfg(feature = "probe")]
pub use defmt as log;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};

#[cfg(not(feature = "probe"))]
pub use log_log as log;

#[cfg(not(feature = "probe"))]
pub trait WhichDebug = ::core::fmt::Debug;
#[cfg(feature = "probe")]
pub trait WhichDebug = ::defmt::Format;

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: ::static_cell::StaticCell<T> = ::static_cell::StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

#[allow(unused_macros)]
macro_rules! general_future_executor {
    ($name:ident, $tyname:ident) => {
        type $tyname = impl ::futures::Future;

        #[embassy_executor::task]
        async fn $name(fut: $tyname) {
            fut.await;
        }
    };
}

#[allow(unused_imports)]
pub(crate) use {general_future_executor, singleton};

pub struct Ticker {
    last_tick: Instant,
    duration: Duration,
}

impl Ticker {
    pub fn every(duration: Duration) -> Self {
        let last_tick = Instant::now();
        Self {
            last_tick,
            duration,
        }
    }

    pub async fn next(&mut self) {
        let now = Instant::now();

        if now.saturating_duration_since(self.last_tick) > self.duration {
            self.last_tick = now;
            return;
        }

        let next_tick = self.last_tick + self.duration;

        Timer::at(next_tick).await;

        self.last_tick = next_tick;
    }
}

pub mod executor_metrics {
    use portable_atomic::{AtomicU64, AtomicUsize};

    pub static WAKEUPS: AtomicUsize = AtomicUsize::new(0);
    pub static AWAKE: AtomicU64 = AtomicU64::new(0);
    pub static SLEEP: AtomicU64 = AtomicU64::new(0);
}

pub struct MeasuringExecutor {
    inner: embassy_executor::raw::Executor,
    not_send: PhantomData<*mut ()>,
    samples: heapless::HistoryBuffer<(u16, u16), 8>,
}

const THREAD_PENDER: usize = usize::MAX;

impl MeasuringExecutor {
    pub fn new() -> Self {
        Self {
            inner: embassy_executor::raw::Executor::new(THREAD_PENDER as *mut ()),
            not_send: PhantomData,
            samples: heapless::HistoryBuffer::new(),
        }
    }

    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());

        loop {
            let start = embassy_time::Instant::now();

            unsafe {
                self.inner.poll();
            }

            let finished = embassy_time::Instant::now();

            unsafe {
                asm!("wfe");
            }

            let now = embassy_time::Instant::now();

            let awake = finished.as_ticks().saturating_sub(start.as_ticks()) as u16;
            let sleeping = now.as_ticks().saturating_sub(finished.as_ticks()) as u16;

            self.samples.write((awake, sleeping));

            let (awake, sleeping) = self.samples.iter().fold(
                (0, 0),
                |(total_awake, total_asleep), (awake, sleeping)| {
                    (total_awake + *awake as u64, total_asleep + *sleeping as u64)
                },
            );

            executor_metrics::WAKEUPS.add(1, portable_atomic::Ordering::Relaxed);
            executor_metrics::AWAKE.add(awake, portable_atomic::Ordering::Relaxed);
            executor_metrics::SLEEP.add(sleeping, portable_atomic::Ordering::Release);
        }
    }
}
