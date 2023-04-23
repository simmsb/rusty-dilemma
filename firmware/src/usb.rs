#[cfg(feature = "probe")]
use defmt as log;

use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
pub use serial_cmd_chan::{COMMANDS_FROM_HOST, COMMANDS_TO_HOST};

pub use device::MAX_PACKET_SIZE;

pub mod device;
pub mod serial_cmd_chan;

pub fn init(spawner: &Spawner, driver: Driver<'static, USB>) {
    log::info!("Initializing usb");

    let usb_state = crate::utils::singleton!(device::State::new());
    let mut builder = device::setup_usb(driver, usb_state);

    serial_cmd_chan::start_static_serial(spawner, &mut builder);

    spawner.must_spawn(device::run_usb(builder));
}
