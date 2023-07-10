use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::PubSubChannel;

use crate::messages::device_to_device::DeviceToDevice;
use crate::messages::transmissions;
use crate::messages::TransmittedMessage;

use super::onewire;

pub static THIS_SIDE_MESSAGE_BUS: PubSubChannel<ThreadModeRawMutex, DeviceToDevice, 16, 6, 6> =
    PubSubChannel::new();
pub static COMMANDS_TO_OTHER_SIDE: Channel<
    ThreadModeRawMutex,
    TransmittedMessage<DeviceToDevice>,
    16,
> = Channel::new();

#[embassy_executor::task]
pub async fn eventer_task() {
    let msg_pub = THIS_SIDE_MESSAGE_BUS.publisher().unwrap();
    let rx_fn = || async { COMMANDS_TO_OTHER_SIDE.recv().await };
    let tx_fn = |e| async {
        msg_pub.publish(e).await;
    };
    transmissions::eventer(
        &onewire::OTHER_SIDE_TX,
        &onewire::OTHER_SIDE_RX,
        rx_fn,
        tx_fn,
    )
    .await;
}
