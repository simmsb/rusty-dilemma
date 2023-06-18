use embassy_executor::Spawner;
use embassy_rp::{
    dma,
    peripherals::PIO1,
    pio::{Common, PioPin, StateMachine},
    Peripheral,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};

use crate::{
    interboard::{self, COMMANDS_FROM_OTHER_SIDE},
    messages::{device_to_device::DeviceToDevice, reliable_msg},
    side,
};

use self::{animation::Animation, animations::DynAnimation};

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

    spawner.must_spawn(runner::rgb_runner(d));
    spawner.must_spawn(command_listener());

    if side::this_side_has_usb() {
        spawner.must_spawn(animation_randomizer());
    }
}

#[embassy_executor::task]
async fn command_listener() {
    let mut sub = COMMANDS_FROM_OTHER_SIDE.subscriber().unwrap();

    loop {
        let cmd = match sub.next_message_pure().await {
            DeviceToDevice::SetAnimation(a) => Command::SetNextAnimation(a),
            DeviceToDevice::SyncAnimation(a) => Command::SyncAnimation(a),
            _ => continue,
        };

        send_cmd(cmd).await;
    }
}

#[embassy_executor::task]
async fn animation_randomizer() {
    loop {
        Timer::after(Duration::from_secs(60 * 30)).await;

        let anim = DynAnimation::random();
        let sync = anim.construct_sync();

        send_cmd(Command::SetNextAnimation(sync.clone())).await;
        interboard::send_msg(reliable_msg(DeviceToDevice::SetAnimation(sync))).await;
    }
}

pub async fn send_cmd(cmd: Command) {
    RGB_CMD_CHANNEL.send(cmd).await
}

pub enum Command {
    SetNextAnimation(animations::AnimationSync),
    SyncAnimation(animations::AnimationSync),
}
