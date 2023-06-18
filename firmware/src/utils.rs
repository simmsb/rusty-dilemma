#[cfg(feature = "probe")]
pub use defmt as log;
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

// taken from https://github.com/randomPoison/take-if/blob/9d3af9f0be7bbe7814e754c3846ce60b372ca3ae/src/lib.rs
pub trait TakeIf {
    /// The type contained in the `Option`.
    type Inner;

    /// Takes value out of the `Option` if `predicate` returns `true`.
    ///
    /// See the [crate-level documentation](./index.html) for more information.
    fn take_if<F: FnOnce(&Self::Inner) -> bool>(&mut self, predicate: F) -> Option<Self::Inner>;
}

impl<T> TakeIf for Option<T> {
    type Inner = T;

    fn take_if<F: FnOnce(&Self::Inner) -> bool>(&mut self, predicate: F) -> Option<Self::Inner> {
        if self.as_ref().map(predicate).unwrap_or(false) {
            self.take()
        } else {
            None
        }
    }
}
