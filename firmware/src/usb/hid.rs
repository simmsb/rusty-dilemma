use embassy_executor::Spawner;
use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_sync::channel::Channel;
use embassy_usb::{class::hid::HidWriter, Builder};
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};

use crate::utils;

type CS = embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

static MOUSE_REPORTS: Channel<CS, MouseReport, 2> = Channel::new();

pub async fn publish_mouse_report(report: MouseReport) {
    MOUSE_REPORTS.send(report).await;
}

#[embassy_executor::task]
async fn hid_writer_task(mut writer: HidWriter<'static, Driver<'static, USB>, 64>) {
    loop {
        let report = MOUSE_REPORTS.recv().await;

        let _ = writer.write_serialize(&report).await;
    }
}

pub fn setup(spawner: &Spawner, builder: &mut Builder<'static, Driver<'static, USB>>) {
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

    spawner.must_spawn(hid_writer_task(mouse_hid_writer));
}
