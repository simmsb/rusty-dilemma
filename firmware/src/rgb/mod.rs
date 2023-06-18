use embassy_executor::Spawner;
use embassy_rp::{
    dma,
    peripherals::PIO1,
    pio::{Common, PioPin, StateMachine},
    Peripheral,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};

pub mod animation;
pub mod animations;
mod driver;
pub mod layout;
mod runner;

pub(super) static RGB_CMD_CHANNEL: Channel<ThreadModeRawMutex, Command, 1> = Channel::new();

pub fn init(
    spawner: &Spawner,
    common: &mut Common<'static, PIO1>,
    sm: StateMachine<'static, PIO1, 0>,
    pin: impl PioPin,
    dma: impl Peripheral<P = impl dma::Channel> + 'static,
) {
    let d = driver::Ws2812::new(common, sm, pin, dma);

    spawner.must_spawn(runner::rgb_runner(d))
}

pub async fn send_cmd(cmd: Command) {
    RGB_CMD_CHANNEL.send(cmd).await
}

pub enum Command {
    SetNextAnimation(animations::AnimationSync),
}
