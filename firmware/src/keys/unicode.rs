use embassy_os_guess::OS;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use usbd_human_interface_device::{device::keyboard::NKROBootKeyboardReport, page::Keyboard};

use crate::usb::{guessed_host_os, hid::publish_keyboard_report};

use super::UnicodeMode;

static UNICODE_MESSAGES: Channel<ThreadModeRawMutex, &'static str, 4> = Channel::new();

pub async fn send_unicode(msg: &'static str) {
    UNICODE_MESSAGES.send(msg).await;
}

#[embassy_executor::task]
pub async fn unicode_task() {
    loop {
        let msg = UNICODE_MESSAGES.receive().await;

        let mode = match guessed_host_os() {
            Some(OS::Linux) => UnicodeMode::Linux,
            _ => UnicodeMode::Mac,
        };

        match mode {
            UnicodeMode::Linux => emit_linux(msg).await,
            UnicodeMode::Mac => emit_mac(msg).await,
        }
    }
}

async fn press_keys(keys: &[Keyboard]) {
    publish_keyboard_report(NKROBootKeyboardReport::new(keys.iter().copied())).await;
}

#[allow(unused)]
async fn tap_keys(keys: &[Keyboard]) {
    press_keys(keys).await;
    publish_keyboard_report(NKROBootKeyboardReport::new([])).await;
}

const HEX_KEYS: [Keyboard; 16] = [
    Keyboard::Keyboard0,
    Keyboard::Keyboard1,
    Keyboard::Keyboard2,
    Keyboard::Keyboard3,
    Keyboard::Keyboard4,
    Keyboard::Keyboard5,
    Keyboard::Keyboard6,
    Keyboard::Keyboard7,
    Keyboard::Keyboard8,
    Keyboard::Keyboard9,
    Keyboard::A,
    Keyboard::B,
    Keyboard::C,
    Keyboard::D,
    Keyboard::E,
    Keyboard::F,
];

fn to_escape(c: char) -> heapless::Vec<Keyboard, 6> {
    let c = c as u32;
    let mut seen_nonzero = false;
    let mut out = heapless::Vec::new();

    let mut f = |n: u32| {
        let nibble = ((c >> n) & 15u32) as usize;
        if seen_nonzero || nibble != 0 {
            out.push(HEX_KEYS[nibble]).unwrap();
            seen_nonzero = true;
        }
    };

    f(20);
    f(16);
    f(12);
    f(8);
    f(4);
    f(0);

    out
}

async fn emit_linux(msg: &str) {
    for c in msg.chars() {
        press_keys(&[Keyboard::LeftControl, Keyboard::LeftShift, Keyboard::U]).await;

        for k in to_escape(c) {
            press_keys(&[Keyboard::LeftControl, Keyboard::LeftShift, Keyboard::U, k]).await;
        }

        press_keys(&[]).await;
    }
}

fn to_escape_surrogate(c: u16) -> heapless::Vec<Keyboard, 6> {
    let mut seen_nonzero = false;
    let mut out = heapless::Vec::new();

    let mut f = |n: u16| {
        let nibble = ((c >> n) & 15u16) as usize;
        if seen_nonzero || nibble != 0 {
            out.push(HEX_KEYS[nibble]).unwrap();
            seen_nonzero = true;
        }
    };

    f(12);
    f(8);
    f(4);
    f(0);

    out
}

async fn emit_mac(msg: &str) {
    press_keys(&[Keyboard::RightAlt]).await;
    embassy_time::Timer::after_millis(50).await;
    for c in msg.encode_utf16() {
        press_keys(&[Keyboard::RightAlt, Keyboard::LeftAlt]).await;
        for k in to_escape_surrogate(c) {
            press_keys(&[Keyboard::RightAlt, Keyboard::LeftAlt, k]).await;
            press_keys(&[Keyboard::RightAlt, Keyboard::LeftAlt]).await;
        }
    }
    press_keys(&[]).await;
}
