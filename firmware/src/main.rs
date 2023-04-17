#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::interrupt;
use embassy_rp::usb::Driver;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

mod event;
mod fw_update;
mod logger;
mod messages;
mod usb;
mod utils;
mod side;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let irq = interrupt::take!(USBCTRL_IRQ);
    let usb_driver = Driver::new(p.USB, irq);

    logger::setup_logger();
    messages::init(&spawner);
    usb::init(&spawner, usb_driver);
    fw_update::init(&spawner, p.WATCHDOG, p.FLASH);

    let mut counter = 0;
    loop {
        counter += 1;
        log::info!("Tick {}", counter);
        Timer::after(Duration::from_secs(1)).await;
    }
}
