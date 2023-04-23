use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::PubSubChannel;
use shared::device_to_device::DeviceToDevice;
use shared::device_to_host::{DeviceToHost, DeviceToHostMsg};
use shared::host_to_device::HostToDeviceMsg;

use crate::side::{self, is_this_side};
use crate::usb;

use super::{reliable_msg, TransmittedMessage};

pub static COMMANDS_TO_OTHER_SIDE: Channel<
    ThreadModeRawMutex,
    TransmittedMessage<DeviceToDevice>,
    4,
> = Channel::new();
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
                    #[cfg(feature = "bootloader")]
                    crate::fw_update::FW_CMD_CHANNEL.send(cmd).await;
                }
            }
        } else {
            COMMANDS_TO_OTHER_SIDE
                .send(reliable_msg(DeviceToDevice::ForwardedFromHost(msg.msg)))
                .await;
        }
    }
}

pub async fn send_to_host(
    TransmittedMessage { msg, timeout }: TransmittedMessage<DeviceToHostMsg>,
) {
    let side = side::get_side();
    let msg = DeviceToHost {
        from_side: side,
        msg,
    };
    if side::this_side_has_usb() {
        let msg = TransmittedMessage { msg, timeout };
        usb::COMMANDS_TO_HOST.send(msg).await;
    } else {
        let msg = DeviceToDevice::ForwardedToHost(msg);
        let msg = TransmittedMessage { msg, timeout };
        COMMANDS_TO_OTHER_SIDE.send(msg).await;
    }
}

pub fn try_send_to_host(
    TransmittedMessage { msg, timeout }: TransmittedMessage<DeviceToHostMsg>,
) -> Option<()> {
    let side = side::get_side();
    let msg = DeviceToHost {
        from_side: side,
        msg,
    };
    if side::this_side_has_usb() {
        let msg = TransmittedMessage { msg, timeout };
        usb::COMMANDS_TO_HOST.try_send(msg).ok()
    } else {
        let msg = DeviceToDevice::ForwardedToHost(msg);
        let msg = TransmittedMessage { msg, timeout };
        COMMANDS_TO_OTHER_SIDE.try_send(msg).ok()
    }
}
