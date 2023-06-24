use atomic_polyfill::AtomicU8;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_sync::channel::Channel;
use embassy_usb::{class::hid::HidWriter, Builder};
use packed_struct::PackedStruct;
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};
use usbd_human_interface_device::device::keyboard::{
    NKROBootKeyboardReport, NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR,
};

use crate::{
    interboard::{self, THIS_SIDE_MESSAGE_BUS},
    messages::{device_to_device::DeviceToDevice, low_latency_msg},
    side, utils,
};

type CS = embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

static MOUSE_REPORTS: Channel<CS, shared::hid::MouseReport, 2> = Channel::new();
static KEYBOARD_REPORTS: Channel<CS, NKROBootKeyboardReport, 2> = Channel::new();

pub async fn publish_mouse_report(report: shared::hid::MouseReport) {
    MOUSE_REPORTS.send(report).await;
    yield_now().await;
}

pub async fn publish_keyboard_report(report: NKROBootKeyboardReport) {
    KEYBOARD_REPORTS.send(report).await;
}

static MOUSE_BUTTON_STATE: AtomicU8 = AtomicU8::new(0);

#[embassy_executor::task]
async fn handle_mouse_clicks() {
    let mut sub = THIS_SIDE_MESSAGE_BUS.subscriber().unwrap();

    loop {
        match sub.next_message_pure().await {
            DeviceToDevice::MousebuttonPress(b) => {
                MOUSE_BUTTON_STATE.fetch_or(b.bit(), atomic_polyfill::Ordering::SeqCst);
                MOUSE_REPORTS.send(shared::hid::MouseReport::default()).await;
            }
            DeviceToDevice::MousebuttonRelease(b) => {
                MOUSE_BUTTON_STATE.fetch_and(!b.bit(), atomic_polyfill::Ordering::SeqCst);
                MOUSE_REPORTS.send(shared::hid::MouseReport::default()).await;
            }
            _ => {}
        }
    }
}

#[embassy_executor::task]
async fn mouse_writer(mut mouse_writer: HidWriter<'static, Driver<'static, USB>, 64>) {
    loop {
        let shared::hid::MouseReport { x, y, wheel, pan } = MOUSE_REPORTS.recv().await;

        let report = MouseReport {
            buttons: MOUSE_BUTTON_STATE.load(atomic_polyfill::Ordering::Relaxed),
            x,
            y,
            wheel,
            pan,
        };

        let _ = mouse_writer.write_serialize(&report).await;
    }
}

#[embassy_executor::task]
async fn keyboard_writer(mut keyboard_writer: HidWriter<'static, Driver<'static, USB>, 64>) {
    loop {
        let report = KEYBOARD_REPORTS.recv().await;
        let _ = keyboard_writer.write(&report.pack().unwrap()).await;
    }
}

#[embassy_executor::task]
async fn interboard_receiver() {
    let mut sub = THIS_SIDE_MESSAGE_BUS.subscriber().unwrap();

    loop {
        let DeviceToDevice::ForwardedToHostMouse(report) = sub.next_message_pure().await else { continue; };

        publish_mouse_report(report).await;
    }
}

pub fn init(spawner: &Spawner, builder: &mut Builder<'static, Driver<'static, USB>>) {
    let mouse_state = utils::singleton!(embassy_usb::class::hid::State::new());
    let keyboard_state = utils::singleton!(embassy_usb::class::hid::State::new());

    let mouse_hid_writer = HidWriter::new(
        builder,
        mouse_state,
        embassy_usb::class::hid::Config {
            report_descriptor: MouseReport::desc(),
            request_handler: None,
            poll_ms: 1,
            max_packet_size: 8,
        },
    );

    let keyboard_hid_writer = HidWriter::new(
        builder,
        keyboard_state,
        embassy_usb::class::hid::Config {
            report_descriptor: NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR,
            request_handler: None,
            poll_ms: 1,
            max_packet_size: 64,
        },
    );

    spawner.must_spawn(mouse_writer(mouse_hid_writer));
    spawner.must_spawn(keyboard_writer(keyboard_hid_writer));
    spawner.must_spawn(interboard_receiver());
    spawner.must_spawn(handle_mouse_clicks());
}

pub async fn send_mouse_hid_to_host(report: shared::hid::MouseReport) {
    if side::this_side_has_usb() {
        publish_mouse_report(report).await;
    } else {
        let msg = DeviceToDevice::ForwardedToHostMouse(report);
        let msg = low_latency_msg(msg);
        interboard::send_msg(msg).await;
    }
}
