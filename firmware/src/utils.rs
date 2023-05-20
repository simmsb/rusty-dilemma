#[cfg(feature = "probe")]
pub use defmt as log;
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
        STATIC_CELL.init_with(move || $val)
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
