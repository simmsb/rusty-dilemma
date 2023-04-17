macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: ::static_cell::StaticCell<T> = ::static_cell::StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}

macro_rules! general_future_executor {
    ($name:ident, $tyname:ident) => {
        type $tyname = impl ::futures::Future;

        #[embassy_executor::task]
        async fn $name(fut: $tyname) {
            fut.await;
        }
    };
}

pub(crate) use {general_future_executor, singleton};
