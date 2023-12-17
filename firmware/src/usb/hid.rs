use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_sync::channel::Channel;
use embassy_usb::{class::hid::HidWriter, Builder};
use num::Integer;
use packed_struct::PackedStruct;
use portable_atomic::{AtomicBool, AtomicU8};
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};
use usbd_human_interface_device::device::keyboard::{
    NKROBootKeyboardReport, NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR,
};

use crate::{
    interboard::{self, THIS_SIDE_MESSAGE_BUS},
    messages::{device_to_device::DeviceToDevice, low_latency_msg},
    side, utils,
};

use super::USBDriver;

type CS = embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

static MOUSE_REPORTS: Channel<CS, shared::hid::MouseReport, 4> = Channel::new();
static KEYBOARD_REPORTS: Channel<CS, NKROBootKeyboardReport, 2> = Channel::new();

pub async fn publish_mouse_report(report: shared::hid::MouseReport) {
    MOUSE_REPORTS.send(report).await;
    yield_now().await;
}

pub async fn publish_keyboard_report(report: NKROBootKeyboardReport) {
    KEYBOARD_REPORTS.send(report).await;
}

static MOUSE_BUTTON_STATE: AtomicU8 = AtomicU8::new(0);
static IS_SCROLLING: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
async fn handle_mouse_clicks() {
    let mut sub = THIS_SIDE_MESSAGE_BUS.subscriber().unwrap();

    loop {
        if let DeviceToDevice::SyncMouseState(b) = sub.next_message_pure().await {
            let buttons: u8 = [
                if b.left() { 0b01 } else { 0 },
                if b.right() { 0b10 } else { 0 },
            ]
            .into_iter()
            .sum();

            MOUSE_BUTTON_STATE.store(buttons, portable_atomic::Ordering::SeqCst);
            IS_SCROLLING.store(b.scrolling(), portable_atomic::Ordering::SeqCst);
            MOUSE_REPORTS
                .send(shared::hid::MouseReport::default())
                .await;
        }
    }
}

const SCROLL_PERIOD: u8 = 12;

#[derive(Default)]
struct ScrollDivider {
    fwd: u8,
    bwd: u8,
}

impl ScrollDivider {
    fn update(&mut self, diff: i8) -> i8 {
        self.fwd = self.fwd.saturating_add_signed(diff);
        self.bwd = self.bwd.saturating_add_signed(-diff);

        let out_fwd;
        (out_fwd, self.fwd) = self.fwd.div_mod_floor(&SCROLL_PERIOD);
        let out_bwd;
        (out_bwd, self.bwd) = self.bwd.div_mod_floor(&SCROLL_PERIOD);

        0i8.saturating_add_unsigned(out_fwd)
            .saturating_sub_unsigned(out_bwd)
    }
}

#[derive(Default)]
struct MovementCoalescer {
    next: i8,
    current: i8,
}

impl MovementCoalescer {
    fn update(&mut self, value: i8) -> bool {
        if let Some(current) = self.current.checked_add(value) {
            self.current = current;
            true
        } else {
            self.next = value;
            false
        }
    }

    fn take(&mut self) -> i8 {
        core::mem::swap(&mut self.current, &mut self.next);
        core::mem::take(&mut self.next)
    }
}

#[embassy_executor::task]
async fn mouse_writer(mut mouse_writer: HidWriter<'static, USBDriver, 64>) {
    let mut vertical_scroll_state = ScrollDivider::default();
    let mut horizontal_scroll_state = ScrollDivider::default();
    let mut x_coalescer = MovementCoalescer::default();
    let mut y_coalescer = MovementCoalescer::default();

    loop {
        let shared::hid::MouseReport { mut x, mut y } = MOUSE_REPORTS.receive().await;
        while x_coalescer.update(x) && y_coalescer.update(y) {
            let Some(shared::hid::MouseReport { x: x_, y: y_ }) = MOUSE_REPORTS.try_receive().ok()
            else {
                break;
            };
            (x, y) = (x_, y_);
        }

        let (x, y, wheel, pan) = if IS_SCROLLING.load(portable_atomic::Ordering::SeqCst) {
            let y = vertical_scroll_state.update(y_coalescer.take());
            let x = horizontal_scroll_state.update(x_coalescer.take());
            (0, 0, y, x)
        } else {
            (x, y, 0, 0)
        };

        let report = MouseReport {
            buttons: MOUSE_BUTTON_STATE.load(portable_atomic::Ordering::SeqCst),
            x,
            y,
            wheel,
            pan,
        };

        let _ = mouse_writer.write_serialize(&report).await;
    }
}

#[embassy_executor::task]
async fn keyboard_writer(mut keyboard_writer: HidWriter<'static, USBDriver, 64>) {
    loop {
        let report = KEYBOARD_REPORTS.receive().await;
        let _ = keyboard_writer.write(&report.pack().unwrap()).await;
    }
}

#[embassy_executor::task]
async fn interboard_receiver() {
    let mut sub = THIS_SIDE_MESSAGE_BUS.subscriber().unwrap();

    loop {
        let DeviceToDevice::ForwardedToHostMouse(report) = sub.next_message_pure().await else {
            continue;
        };

        publish_mouse_report(report).await;
    }
}

pub fn init(spawner: &Spawner, builder: &mut Builder<'static, USBDriver>) {
    let mouse_state = utils::singleton!(embassy_usb::class::hid::State::new());
    let keyboard_state = utils::singleton!(embassy_usb::class::hid::State::new());

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

    let keyboard_hid_writer = HidWriter::new(
        builder,
        keyboard_state,
        embassy_usb::class::hid::Config {
            report_descriptor: NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR,
            request_handler: None,
            poll_ms: 10,
            max_packet_size: 64,
        },
    );

    spawner.must_spawn(mouse_writer(mouse_hid_writer));
    spawner.must_spawn(keyboard_writer(keyboard_hid_writer));
    spawner.must_spawn(handle_mouse_clicks());

    if side::this_side_has_usb() && side::is_this_side(shared::side::KeyboardSide::Left) {
        spawner.must_spawn(interboard_receiver());
    }
}

pub async fn send_mouse_hid_to_host(report: shared::hid::MouseReport) {
    if side::this_side_has_usb() {
        publish_mouse_report(report.clone()).await;
    }
    let msg = DeviceToDevice::ForwardedToHostMouse(report);
    let msg = low_latency_msg(msg);
    interboard::send_msg(msg).await;
}
