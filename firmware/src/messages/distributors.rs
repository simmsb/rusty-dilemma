use shared::device_to_device::DeviceToDevice;
use shared::device_to_host::{DeviceToHost, DeviceToHostMsg};
use shared::host_to_device::HostToDeviceMsg;

use crate::side;
use crate::utils::log;
use crate::{interboard, usb};

use super::{reliable_msg, unreliable_msg, TransmittedMessage};

#[embassy_executor::task]
pub async fn from_usb_distributor() {
    let mut sub = crate::usb::COMMANDS_FROM_HOST.subscriber().unwrap();

    loop {
        let msg = sub.next_message_pure().await;

        if msg.targets_side(side::get_side()) {
            handle_from_host(msg.msg.clone()).await;
        }
        if msg.targets_side(side::get_other_side()) {
            interboard::send_msg(reliable_msg(DeviceToDevice::ForwardedFromHost(msg.msg))).await;
        }
    }
}

async fn handle_from_host(msg: HostToDeviceMsg) {
    match msg {
        HostToDeviceMsg::FWCmd(_cmd) => {
            #[cfg(feature = "bootloader")]
            crate::fw_update::FW_CMD_CHANNEL.send(_cmd).await;
        }
    }
}

#[embassy_executor::task]
pub async fn from_other_side_distributor() {
    let mut sub = crate::interboard::COMMANDS_FROM_OTHER_SIDE
        .subscriber()
        .unwrap();

    loop {
        let msg = sub.next_message_pure().await;

        match msg {
            DeviceToDevice::Ping => {
                log::info!("Got a ping");
                interboard::send_msg(reliable_msg(DeviceToDevice::Pong)).await;
            }
            DeviceToDevice::Pong => {
                log::info!("Got a pong");
            }
            DeviceToDevice::ForwardedToHost(msg) => {
                usb::send_msg(unreliable_msg(msg)).await;
            }
            DeviceToDevice::ForwardedFromHost(msg) => {
                handle_from_host(msg).await;
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum MessageProvenance {
    Origin,
    Forwarded,
}

pub async fn send_to_host(
    TransmittedMessage { msg, timeout }: TransmittedMessage<DeviceToHostMsg>,
    provenance: MessageProvenance,
) {
    let side = side::get_side();
    let msg = DeviceToHost {
        from_side: side,
        msg,
    };
    if side::this_side_has_usb() {
        let msg = TransmittedMessage { msg, timeout };
        usb::send_msg(msg).await;
    } else if provenance == MessageProvenance::Origin {
        let msg = DeviceToDevice::ForwardedToHost(msg);
        let msg = TransmittedMessage { msg, timeout };
        interboard::send_msg(msg).await;
    }
}

pub fn try_send_to_host(
    TransmittedMessage { msg, timeout }: TransmittedMessage<DeviceToHostMsg>,
    provenance: MessageProvenance,
) -> Option<()> {
    let side = side::get_side();
    let msg = DeviceToHost {
        from_side: side,
        msg,
    };
    if side::this_side_has_usb() {
        let msg = TransmittedMessage { msg, timeout };
        usb::try_send_msg(msg).ok()
    } else if provenance == MessageProvenance::Origin {
        let msg = DeviceToDevice::ForwardedToHost(msg);
        let msg = TransmittedMessage { msg, timeout };
        interboard::try_send_msg(msg).ok()
    } else {
        // if we get here it means both sides have no usb connection
        Some(())
    }
}
