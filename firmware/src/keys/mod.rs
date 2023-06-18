use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::{
    gpio::{Input, Output},
    peripherals::{PIN_26, PIN_27, PIN_28, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, pubsub::PubSubChannel};
use embassy_time::Duration;
use keyberon::{key_code::KeyCode, layout::Event};
use packed_struct::PrimitiveEnum;
use usbd_human_interface_device::device::keyboard::NKROBootKeyboardReport;

use crate::{
    interboard,
    messages::{device_to_device::DeviceToDevice, reliable_msg},
    side,
    usb::hid::publish_keyboard_report,
    utils::Ticker,
};

use self::{chord::ChordingEngine, layout::LAYERS};

pub mod chord;
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
    let mut ticker = Ticker::every(Duration::from_hz(500));
    let matrix_events = MATRIX_EVENTS.publisher().unwrap();

    let is_right = side::get_side().is_right();

    loop {
        for evt in scanner.scan() {
            let evt = if is_right {
                evt.transform(|x, y| (x, 9 - y))
            } else {
                evt
            };

            matrix_events.publish(evt).await;
        }

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn matrix_processor() {
    let mut sub = MATRIX_EVENTS.subscriber().unwrap();
    let key_events = KEY_EVENTS.publisher().unwrap();
    let mut chorder = ChordingEngine::new(layout::chorder());
    let mut ticker = Ticker::every(Duration::from_hz(1000));

    loop {
        match select(ticker.next(), sub.next_message_pure()).await {
            embassy_futures::select::Either::Second(evt) => {
                //key_events.publish(evt).await;
                let evts = chorder.process(evt);
                for evt in evts {
                    key_events.publish(evt).await;
                }
            }
            embassy_futures::select::Either::First(_) => {
                let keys = chorder.tick();
                for (x, y) in keys {
                    key_events
                        .publish(keyberon::layout::Event::Press(x, y))
                        .await;
                }
            }
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
        interboard::send_msg(reliable_msg(evt)).await;
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

#[embassy_executor::task]
async fn key_event_processor() {
    let mut sub = KEY_EVENTS.subscriber().unwrap();
    let mut layout = keyberon::layout::Layout::new(&LAYERS);
    let mut state = heapless::Vec::<KeyCode, 24>::new();
    let mut ticker = Ticker::every(Duration::from_hz(1000));

    loop {
        match select(ticker.next(), sub.next_message_pure()).await {
            embassy_futures::select::Either::Second(evt) => {
                // crate::utils::log::info!("evt: {:?}", evt);

                layout.event(evt);
            }
            embassy_futures::select::Either::First(_) => {
                let _ = layout.tick();
            }
        }

        let new_state = heapless::Vec::<_, 24>::from_iter(layout.keycodes());

        if new_state != state {
            state = new_state;

            publish_keyboard_report(NKROBootKeyboardReport::new(state.iter().filter_map(|k| {
                usbd_human_interface_device::page::Keyboard::from_primitive(*k as u8)
            })))
            .await;
        }
    }
}

pub fn init(spawner: &Spawner, scanner: ScannerInstance<'static>) {
    spawner.must_spawn(matrix_processor());
    spawner.must_spawn(matrix_scanner(scanner));
    if side::this_side_has_usb() {
        spawner.must_spawn(receive_events_from_other_side());
        spawner.must_spawn(key_event_processor());
    } else {
        spawner.must_spawn(send_events_to_side_with_usb());
    }
}
