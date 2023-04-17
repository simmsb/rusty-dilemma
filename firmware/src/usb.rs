pub mod device;
pub mod serial_cmd_chan;

pub use device::MAX_PACKET_SIZE;
use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
pub use serial_cmd_chan::{COMMANDS_FROM_HOST, COMMANDS_TO_HOST};

pub fn init(spawner: &Spawner, driver: Driver<'static, USB>) {
    let usb_state = crate::utils::singleton!(device::State::new());
    let mut builder = device::setup_usb(driver, usb_state);

    serial_cmd_chan::start_static_serial(spawner, &mut builder);

    spawner.must_spawn(device::run_usb(builder));
}
