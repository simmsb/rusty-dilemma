use core::num::Wrapping;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, pubsub::PubSubChannel};
use embassy_time::Duration;
use num::integer::Roots;
use serde::{Deserialize, Serialize};

use crate::{
    flash, interboard::THIS_SIDE_MESSAGE_BUS, keys::KEY_EVENTS,
    messages::device_to_device::DeviceToDevice, utils,
};

static CURRENT_METRICS: Mutex<ThreadModeRawMutex, Metrics> = Mutex::new(Metrics::default());

pub static METRIC_UPDATES: PubSubChannel<ThreadModeRawMutex, Metrics, 1, 4, 1> =
    PubSubChannel::new();

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Metrics {
    pub keys_pressed: Wrapping<usize>,
    pub trackpad_distance: Wrapping<usize>,
}

impl Metrics {
    const fn default() -> Self {
        Self {
            keys_pressed: Wrapping(0),
            trackpad_distance: Wrapping(0),
        }
    }
}

pub async fn init(spawner: &Spawner) {
    if let Some(m) = flash::get::<Metrics>().await {
        utils::log::info!("Loaded up metrics with: {:?}", m);
        *CURRENT_METRICS.lock().await = m.clone();
        push_update(m);
    }

    spawner.must_spawn(metrics_syncer());
    spawner.must_spawn(key_counter());
    spawner.must_spawn(mouse_counter());
}

fn push_update(m: Metrics) {
    let p = METRIC_UPDATES.immediate_publisher();
    p.publish_immediate(m);
}

pub async fn request_sync() {
    push_update(CURRENT_METRICS.lock().await.clone());
}

#[embassy_executor::task]
async fn mouse_counter() {
    let mut sub = THIS_SIDE_MESSAGE_BUS.subscriber().unwrap();

    loop {
        let DeviceToDevice::ForwardedToHostMouse(report) = sub.next_message_pure().await else {
            continue;
        };

        let x = report.x as u16;
        let y = report.y as u16;

        let distance = (x * x + y * y).sqrt();

        let mut m = CURRENT_METRICS.lock().await;
        m.trackpad_distance += distance as usize;

        push_update(m.clone());
    }
}

#[embassy_executor::task]
async fn key_counter() {
    let mut sub = KEY_EVENTS.subscriber().unwrap();

    loop {
        let k = sub.next_message_pure().await;
        if !k.is_press() {
            continue;
        }

        let mut m = CURRENT_METRICS.lock().await;
        m.keys_pressed += 1;

        push_update(m.clone());
    }
}

#[embassy_executor::task]
async fn metrics_syncer() {
    let mut tick = embassy_time::Ticker::every(Duration::from_secs(60 * 5));
    let mut last = Metrics::default();

    loop {
        tick.next().await;

        let current = CURRENT_METRICS.lock().await.clone();

        if current != last {
            let _ = flash::set(&current).await;

            utils::log::info!("Synced metrics: {:?}", current);
        }

        last = current;
    }
}
