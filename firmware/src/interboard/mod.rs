use embassy_executor::Spawner;
use embassy_rp::{
    peripherals::PIO0,
    pio::{Common, PioPin},
    Peripheral, gpio::{Output, AnyPin},
};
use embassy_sync::channel::TrySendError;
use shared::device_to_device::DeviceToDevice;

use crate::messages::TransmittedMessage;

pub use self::channel::COMMANDS_FROM_OTHER_SIDE;
use self::onewire::SM;
pub mod channel;
pub mod onewire;

pub fn init(
    spawner: &Spawner,
    common: &mut Common<'static, PIO0>,
    tx_sm: SM<0>,
    rx_sm: SM<1>,
    pin: impl Peripheral<P = impl PioPin + 'static> + 'static,
) {
    onewire::init(spawner, common, tx_sm, rx_sm, pin);

    spawner.must_spawn(channel::eventer_task());
}

pub async fn send_msg(msg: TransmittedMessage<DeviceToDevice>) {
    channel::COMMANDS_TO_OTHER_SIDE.send(msg).await;
}

pub fn try_send_msg(
    msg: TransmittedMessage<DeviceToDevice>,
) -> Result<(), TrySendError<TransmittedMessage<DeviceToDevice>>> {
    channel::COMMANDS_TO_OTHER_SIDE.try_send(msg)
}
