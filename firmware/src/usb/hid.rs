use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_sync::channel::Channel;
use embassy_usb::{class::hid::HidWriter, Builder};
use shared::{device_to_device::DeviceToDevice, hid::HidReport};
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};

use crate::{
    interboard::{self, COMMANDS_FROM_OTHER_SIDE},
    messages::low_latency_msg,
    side, utils,
};

type CS = embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

static REPORTS: Channel<CS, HidReport, 2> = Channel::new();

pub async fn publish_report(report: HidReport) {
    REPORTS.send(report).await;
    yield_now().await;
}

#[embassy_executor::task]
async fn hid_writer(mut writer: HidWriter<'static, Driver<'static, USB>, 64>) {
    loop {
        let report = REPORTS.recv().await;

        match report {
            HidReport::Mouse(shared::hid::MouseReport { x, y, wheel, pan }) => {
                let report = MouseReport {
                    buttons: 0,
                    x,
                    y,
                    wheel,
                    pan,
                };

                let _ = writer.write_serialize(&report).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn interboard_receiver() {
    let mut sub = COMMANDS_FROM_OTHER_SIDE.subscriber().unwrap();

    loop {
        let DeviceToDevice::ForwardedToHostHid(report) = sub.next_message_pure().await else { continue; };

        publish_report(report).await;
    }
}

pub fn init(spawner: &Spawner, builder: &mut Builder<'static, Driver<'static, USB>>) {
    let mouse_state = utils::singleton!(embassy_usb::class::hid::State::new());

    let mouse_hid_writer = HidWriter::new(
        builder,
        mouse_state,
        embassy_usb::class::hid::Config {
            report_descriptor: MouseReport::desc(),
            request_handler: None,
            poll_ms: 10,
            max_packet_size: 8,
        },
    );

    spawner.must_spawn(hid_writer(mouse_hid_writer));

    if side::this_side_has_usb() {
        spawner.must_spawn(interboard_receiver());
    }
}

pub async fn send_hid_to_host(report: HidReport) {
    if side::this_side_has_usb() {
        publish_report(report).await;
    } else {
        let msg = DeviceToDevice::ForwardedToHostHid(report);
        let msg = low_latency_msg(msg);
        interboard::send_msg(msg).await;
    }
}
