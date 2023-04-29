use embassy_executor::Spawner;
use embassy_rp::{
    gpio::AnyPin,
    pio::{Sm0, Sm1},
};
use embassy_sync::channel::TrySendError;
use shared::device_to_device::DeviceToDevice;

use crate::messages::TransmittedMessage;

use self::onewire::SM;
pub use self::channel::COMMANDS_FROM_OTHER_SIDE;
pub mod channel;
pub mod onewire;

pub fn init(spawner: &Spawner, tx_sm: SM<Sm0>, rx_sm: SM<Sm1>, pin: AnyPin) {
    onewire::init(spawner, tx_sm, rx_sm, pin);

    spawner.must_spawn(channel::eventer_task());
}

pub async fn send_msg(msg: TransmittedMessage<DeviceToDevice>) {
    channel::COMMANDS_TO_OTHER_SIDE.send(msg).await;
}

pub fn try_send_msg(msg: TransmittedMessage<DeviceToDevice>) -> Result<(), TrySendError<TransmittedMessage<DeviceToDevice>>> {
    channel::COMMANDS_TO_OTHER_SIDE.try_send(msg)
}
