use core::{future::poll_fn, task::Poll};

#[cfg(feature = "probe")]
pub use defmt as log;
use embassy_time::{Duration, Instant, Timer};
use futures::Future;
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

        let mut next_tick = self.last_tick + self.duration;

        // skip ticks until we're within one duration of now
        while next_tick + self.duration < now {
            next_tick += self.duration;
        }

        Timer::at(next_tick).await;

        self.last_tick = next_tick;
    }
}
