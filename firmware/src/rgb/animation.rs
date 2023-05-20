use cichlid::ColorRGB;
use embassy_time::Duration;

use super::layout::Light;

pub trait Animation: Default {
    const TICK_RATE: Duration;

    fn tick_rate(&self) -> Duration {
        Self::TICK_RATE
    }

    fn tick(&mut self);
    fn render(&self, light: &Light) -> ColorRGB;
}
