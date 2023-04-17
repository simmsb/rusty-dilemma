use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::PubSubChannel;
use embassy_time::Duration;
use shared::device_to_device::DeviceToDevice;
use shared::host_to_device::HostToDeviceMsg;

use crate::side::is_this_side;

pub static COMMANDS_TO_OTHER_SIDE: Channel<ThreadModeRawMutex, (DeviceToDevice, Duration), 4> =
    Channel::new();
pub static COMMANDS_FROM_HOST: PubSubChannel<ThreadModeRawMutex, DeviceToDevice, 4, 4, 1> =
    PubSubChannel::new();

#[embassy_executor::task]
pub async fn from_usb_distributor() {
    let mut sub = crate::usb::COMMANDS_FROM_HOST.subscriber().unwrap();

    loop {
        let msg = sub.next_message_pure().await;

        if is_this_side(msg.target_side) {
            match msg.msg {
                HostToDeviceMsg::FWCmd(cmd) => {
                    crate::fw_update::FW_CMD_CHANNEL.send(cmd).await;
                }
            }
        } else {
            COMMANDS_TO_OTHER_SIDE
                .send((
                    DeviceToDevice::Forwarded(msg.msg),
                    Duration::from_millis(50),
                ))
                .await;
        }
    }
}
