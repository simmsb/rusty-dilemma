use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_sync::channel::TrySendError;

use shared::device_to_host::DeviceToHost;

pub use channel::COMMANDS_FROM_HOST;
pub use device::MAX_PACKET_SIZE;
pub use hid::publish_report;

use crate::messages::TransmittedMessage;
use crate::utils::log;

pub mod channel;
pub mod device;
pub mod hid;
pub mod picotool;

pub fn init(spawner: &Spawner, driver: Driver<'static, USB>) {
    log::info!("Initializing usb");

    let mut builder = device::init_usb(driver);

    channel::init(spawner, &mut builder);
    picotool::init(&mut builder);
    hid::init(spawner, &mut builder);

    spawner.must_spawn(device::run_usb(builder));
}

pub async fn send_msg(msg: TransmittedMessage<DeviceToHost>) {
    if let Err(TrySendError::Full(msg)) = try_send_msg(msg) {
        // this is a bit of a hack, for messages destined over the usb serial we
        // allow non-reliable messages to be dropped here if the queue is full,
        // as it likely means that the serial channel is not open
        //
        // we shouldn't be sending anything other than unreliable messages over
        // here anyway
        if msg.timeout.is_none() {
            return;
        }

        channel::COMMANDS_TO_HOST.send(msg).await;
    }
}

pub fn try_send_msg(
    msg: TransmittedMessage<DeviceToHost>,
) -> Result<(), TrySendError<TransmittedMessage<DeviceToHost>>> {
    channel::COMMANDS_TO_HOST.try_send(msg)
}
