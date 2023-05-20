use embassy_executor::Spawner;
use embassy_rp::{
    dma::Channel,
    peripherals::PIO1,
    pio::{Common, PioPin, StateMachine},
    Peripheral,
};

pub mod animation;
pub mod animations;
mod driver;
pub mod layout;
mod runner;

pub fn init(
    spawner: &Spawner,
    common: &mut Common<'static, PIO1>,
    sm: StateMachine<'static, PIO1, 0>,
    pin: impl PioPin,
    dma: impl Peripheral<P = impl Channel> + 'static,
) {
    let d = driver::Ws2812::new(common, sm, pin, dma);

    spawner.must_spawn(runner::rgb_runner(d))
}
