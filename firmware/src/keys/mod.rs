use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::{
    gpio::{Input, Output},
    peripherals::{PIN_26, PIN_27, PIN_28, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9},
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, pubsub::PubSubChannel,
};
use embassy_time::Duration;
use keyberon::{key_code::KeyCode, layout::Event};
use packed_struct::PrimitiveEnum;
use usbd_human_interface_device::device::keyboard::NKROBootKeyboardReport;

use crate::{
    interboard::{self, THIS_SIDE_MESSAGE_BUS},
    messages::{
        device_to_device::{DeviceToDevice, MouseState},
        reliable_msg,
    },
    side,
    usb::hid::publish_keyboard_report,
    utils::Ticker,
};

use self::{chord::ChordingEngine, layout::LAYERS};

#[derive(Clone, Copy)]
pub enum UnicodeMode {
    Linux,
    Mac,
}

#[derive(Clone, Copy)]
pub enum CustomEvent {
    MouseLeft,
    MouseRight,
    MouseScroll,
    TypeUnicode(&'static str, UnicodeMode),
}

pub mod chord;
pub mod layout;
pub mod scan;
mod unicode;

/// Raw matrix presses and releases
pub static MATRIX_EVENTS: PubSubChannel<ThreadModeRawMutex, keyberon::layout::Event, 4, 4, 1> =
    PubSubChannel::new();

/// Chord-processed events
pub static KEY_EVENTS: PubSubChannel<ThreadModeRawMutex, keyberon::layout::Event, 4, 4, 2> =
    PubSubChannel::new();

static KEYS_TO_OTHER_SIDE: Channel<ThreadModeRawMutex, keyberon::layout::Event, 4> = Channel::new();

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
                    KEYS_TO_OTHER_SIDE.send(evt).await;
                }
            }
            embassy_futures::select::Either::First(_) => {
                let keys = chorder.tick();
                for (x, y) in keys {
                    let evt = keyberon::layout::Event::Press(x, y);
                    key_events.publish(evt).await;
                    KEYS_TO_OTHER_SIDE.send(evt).await;
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn send_events_to_other_side() {
    loop {
        let evt = KEYS_TO_OTHER_SIDE.recv().await;
        let evt = match evt {
            Event::Press(x, y) => DeviceToDevice::KeyPress(x, y),
            Event::Release(x, y) => DeviceToDevice::KeyRelease(x, y),
        };
        interboard::send_msg(reliable_msg(evt)).await;
    }
}

#[embassy_executor::task]
async fn receive_events_from_other_side() {
    let mut sub = crate::interboard::THIS_SIDE_MESSAGE_BUS
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
    let msg_bus_pub = THIS_SIDE_MESSAGE_BUS.publisher().unwrap();
    let mut sub = KEY_EVENTS.subscriber().unwrap();
    let mut layout = keyberon::layout::Layout::new(&LAYERS);
    let mut state = heapless::Vec::<KeyCode, 24>::new();
    let mut ticker = Ticker::every(Duration::from_hz(1000));
    let mut mouse_state = MouseState::new();

    loop {
        match select(ticker.next(), sub.next_message_pure()).await {
            embassy_futures::select::Either::Second(evt) => {
                // crate::utils::log::info!("evt: {:?}", evt);

                layout.event(evt);
            }
            embassy_futures::select::Either::First(_) => {
                let cevent = layout.tick();
                if let Some((evt, is_press)) = match cevent {
                    keyberon::layout::CustomEvent::NoEvent => None,
                    keyberon::layout::CustomEvent::Press(m) => Some((*m, true)),
                    keyberon::layout::CustomEvent::Release(m) => Some((*m, false)),
                } {
                    match evt {
                        CustomEvent::MouseLeft => mouse_state.set_left(is_press),
                        CustomEvent::MouseRight => mouse_state.set_right(is_press),
                        CustomEvent::MouseScroll => mouse_state.set_scrolling(is_press),
                        CustomEvent::TypeUnicode(msg, mode) => {
                            if !is_press {
                                unicode::send_unicode(msg, mode).await;
                            }
                        }
                    }

                    let evt = DeviceToDevice::SyncMouseState(mouse_state);

                    interboard::send_msg(reliable_msg(evt.clone())).await;
                    msg_bus_pub.publish(evt).await;
                }
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
    spawner.must_spawn(send_events_to_other_side());
    if side::this_side_has_usb() {
        spawner.must_spawn(receive_events_from_other_side());
        spawner.must_spawn(key_event_processor());
        spawner.must_spawn(unicode::unicode_task());
    }
}
