use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Output},
    peripherals::{PIN_26, PIN_27, PIN_28, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9},
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    pubsub::{PubSubChannel, Subscriber},
};
use embassy_time::Duration;
use keyberon::{chording::Chording, layout::Event};
use shared::device_to_device::DeviceToDevice;

use crate::{
    interboard,
    messages::low_latency_msg,
    side,
    utils::{self, Ticker},
};

pub mod layout;
pub mod scan;

/// Raw matrix presses and releases
pub static MATRIX_EVENTS: PubSubChannel<ThreadModeRawMutex, keyberon::layout::Event, 4, 4, 1> =
    PubSubChannel::new();

/// Chord-processed events
pub static KEY_EVENTS: PubSubChannel<ThreadModeRawMutex, keyberon::layout::Event, 4, 4, 2> =
    PubSubChannel::new();

pub type ScannerInstance<'a> = scan::Scanner<
    (
        Input<'a, PIN_4>,
        Input<'a, PIN_5>,
        Input<'a, PIN_27>,
        Input<'a, PIN_26>,
    ),
    (
        Output<'a, PIN_8>,
        Output<'a, PIN_9>,
        Output<'a, PIN_7>,
        Output<'a, PIN_6>,
        Output<'a, PIN_28>,
    ),
>;

#[embassy_executor::task]
async fn matrix_scanner(mut scanner: ScannerInstance<'static>) {
    let mut ticker = Ticker::every(Duration::from_hz(100));
    let matrix_events = MATRIX_EVENTS.publisher().unwrap();

    let is_right = side::get_side().is_right();

    loop {
        for evt in scanner.scan() {
            let evt = if is_right {
                evt.transform(|x, y| (9 - x, y))
            } else {
                evt
            };
            utils::log::info!("evt: {:?}", evt);
            matrix_events.publish(evt).await;
        }

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn matrix_processor() {
    let mut sub = MATRIX_EVENTS.subscriber().unwrap();
    let key_events = KEY_EVENTS.publisher().unwrap();
    let mut chorder = Chording::new(&layout::CHORDS);

    loop {
        let evt = sub.next_message_pure().await;

        let evts = heapless::Vec::from_iter(core::iter::once(evt));
        let evts = chorder.tick(evts);

        for evt in evts {
            key_events.publish(evt).await;
        }
    }
}

#[embassy_executor::task]
async fn send_events_to_side_with_usb() {
    let mut sub = KEY_EVENTS.subscriber().unwrap();

    loop {
        let evt = sub.next_message_pure().await;
        let evt = match evt {
            Event::Press(x, y) => DeviceToDevice::KeyPress(x, y),
            Event::Release(x, y) => DeviceToDevice::KeyRelease(x, y),
        };
        interboard::send_msg(low_latency_msg(evt)).await;
    }
}

#[embassy_executor::task]
async fn receive_events_from_other_side() {
    let mut sub = crate::interboard::COMMANDS_FROM_OTHER_SIDE
        .subscriber()
        .unwrap();
    let key_events = KEY_EVENTS.publisher().unwrap();

    loop {
        let evt = match sub.next_message_pure().await {
            DeviceToDevice::KeyPress(x, y) => Event::Press(x, y),
            DeviceToDevice::KeyRelease(x, y) => Event::Release(x, y),
            _ => {
                continue;
            }
        };

        key_events.publish(evt).await;
    }
}

pub fn init(spawner: &Spawner, scanner: ScannerInstance<'static>) {
    spawner.must_spawn(matrix_processor());
    spawner.must_spawn(matrix_scanner(scanner));
    if side::this_side_has_usb() {
        spawner.must_spawn(receive_events_from_other_side());
    } else {
        spawner.must_spawn(send_events_to_side_with_usb());
    }
}
