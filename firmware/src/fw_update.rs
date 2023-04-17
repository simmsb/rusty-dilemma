pub mod updater_task;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{FLASH, WATCHDOG};
pub use updater_task::FW_CMD_CHANNEL;

pub fn init(spawner: &Spawner, watchdog: WATCHDOG, flash: FLASH) {
    spawner.must_spawn(updater_task::updater_task(watchdog, flash));
}
