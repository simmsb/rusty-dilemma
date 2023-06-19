use embassy_time::Duration;

use crate::rgb::animation::Animation;

pub struct Null;

impl Animation for Null {
    type SyncMessage = ();

    fn tick_rate(&self) -> embassy_time::Duration {
        Duration::from_hz(1)
    }

    fn tick(&mut self) {}

    fn render(&self, _light: &crate::rgb::layout::Light) -> cichlid::ColorRGB {
        cichlid::ColorRGB::Black
    }

    fn construct_sync(&self) -> Self::SyncMessage {
        ()
    }

    fn sync(&mut self, _sync: Self::SyncMessage) {}

    fn new_from_sync(_sync: Self::SyncMessage) -> Self {
        Null
    }
}
