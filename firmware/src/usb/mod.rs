use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
pub use channel::COMMANDS_FROM_HOST;

pub use device::MAX_PACKET_SIZE;
use embassy_sync::channel::TrySendError;
use shared::device_to_host::DeviceToHost;

use crate::messages::TransmittedMessage;
use crate::utils::log;

pub mod device;
pub mod channel;

pub fn init(spawner: &Spawner, driver: Driver<'static, USB>) {
    log::info!("Initializing usb");

    let usb_state = crate::utils::singleton!(device::State::new());
    let mut builder = device::setup_usb(driver, usb_state);

    channel::start_static_serial(spawner, &mut builder);

    spawner.must_spawn(device::run_usb(builder));
}

pub async fn send_msg(msg: TransmittedMessage<DeviceToHost>) {
    channel::COMMANDS_TO_HOST.send(msg).await;
}

pub fn try_send_msg(msg: TransmittedMessage<DeviceToHost>) -> Result<(), TrySendError<TransmittedMessage<DeviceToHost>>> {
    channel::COMMANDS_TO_HOST.try_send(msg)
}
