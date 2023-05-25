use embassy_rp::peripherals::USB;
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, Config};

use crate::utils;

pub const MAX_PACKET_SIZE: u16 = 64;

pub fn init_usb<'d, D: Driver<'d>>(driver: D) -> Builder<'d, D> {
    let mut config = Config::new(0x2e8a, 0x000a);
    config.manufacturer = Some("Ben Simms");
    config.product = Some("Dilemma");
    config.serial_number = None;
    config.max_power = 500;
    config.max_packet_size_0 = MAX_PACKET_SIZE as u8;

    Builder::new(
        driver,
        config,
        &mut utils::singleton!([0; 256])[..],
        &mut utils::singleton!([0; 256])[..],
        &mut utils::singleton!([0; 256])[..],
        &mut utils::singleton!([0; 128])[..],
    )
}

#[embassy_executor::task]
pub async fn run_usb(builder: Builder<'static, embassy_rp::usb::Driver<'static, USB>>) {
    let mut device = builder.build();
    device.run().await;
}
