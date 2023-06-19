use core::{fmt::Debug, hash::Hash};

use cichlid::ColorRGB;
use embassy_time::Duration;
use serde::{de::DeserializeOwned, Serialize};

use super::layout::Light;

pub trait Animation {
    type SyncMessage: DeserializeOwned + Serialize + Eq + PartialEq + Hash + Clone + Debug;

    fn tick_rate(&self) -> Duration;
    fn tick(&mut self);
    fn render(&self, light: &Light) -> ColorRGB;

    fn construct_sync(&self) -> Self::SyncMessage;
    fn sync(&mut self, sync: Self::SyncMessage);
    fn new_from_sync(sync: Self::SyncMessage) -> Self;
}
