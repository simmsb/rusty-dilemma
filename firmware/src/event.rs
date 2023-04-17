use core::fmt::Debug;

use embassy_sync::signal::Signal;

pub struct Event(Signal<embassy_sync::blocking_mutex::raw::ThreadModeRawMutex, ()>);

impl Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Event").finish()
    }
}

impl Event {
    pub const fn new() -> Self {
        Self(Signal::new())
    }

    pub async fn wait(&self) {
        self.0.wait().await;
        self.0.reset();
    }

    pub fn set(&self) {
        self.0.signal(());
    }
}
