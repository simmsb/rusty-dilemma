use core::num::Wrapping;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, pubsub::PubSubChannel};
use embassy_time::Duration;
use serde::{Deserialize, Serialize};

use crate::{flash, keys::KEY_EVENTS, utils};

static CURRENT_METRICS: Mutex<ThreadModeRawMutex, Metrics> = Mutex::new(Metrics::default());

pub static METRIC_UPDATES: PubSubChannel<ThreadModeRawMutex, Metrics, 1, 4, 1> =
    PubSubChannel::new();

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Metrics {
    keys_pressed: Wrapping<usize>,
}

impl Metrics {
    const fn default() -> Self {
        Self {
            keys_pressed: Wrapping(0),
        }
    }
}

pub async fn init(spawner: &Spawner) {
    if let Some(m) = flash::get::<Metrics>().await {
        *CURRENT_METRICS.lock().await = m;
    }

    spawner.must_spawn(metrics_syncer());
    spawner.must_spawn(key_counter());
}

fn push_update(m: Metrics) {
    let p = METRIC_UPDATES.immediate_publisher();
    p.publish_immediate(m);
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
