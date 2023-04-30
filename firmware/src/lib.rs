#![no_std]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait, trait_alias, async_fn_in_trait)]

use embassy_executor::Spawner;
use embassy_rp::interrupt;
use embassy_rp::peripherals::PIN_25;
use embassy_rp::usb::Driver;
use embassy_rp::{gpio::Output, pio::PioPeripheral};
use embassy_time::{Duration, Timer};
use shared::side::KeyboardSide;

#[cfg(not(feature = "probe"))]
use panic_reset as _;
#[cfg(feature = "probe")]
use {defmt_rtt as _, panic_probe as _};

use utils::log;

use crate::messages::reliable_msg;

pub mod event;
#[cfg(feature = "bootloader")]
pub mod fw_update;
pub mod logger;
pub mod messages;
pub mod interboard;
pub mod side;
pub mod usb;
pub mod utils;
pub mod trackpad;

pub static VERSION: &str = "0.1.0";

fn detect_usb() -> bool {
    let regs = embassy_rp::pac::USBCTRL_REGS;
    let connected = unsafe { regs.sie_status().read().connected() };
    log::info!("Usb connected? {}", connected);
    connected
}

#[embassy_executor::task]
async fn blinky(mut pin: Output<'static, PIN_25>) {
    loop {
        pin.set_high();
        Timer::after(Duration::from_secs(1)).await;

        pin.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}

pub async fn main(spawner: Spawner, side: KeyboardSide) {
    let p = embassy_rp::init(Default::default());

    log::info!("Just a whisper. I hear it in my ghost.");

    // not sure if this makes the usb detection happier
    Timer::after(Duration::from_micros(100)).await;

    side::init(side, detect_usb());

    if side::this_side_has_usb() {
        let irq = interrupt::take!(USBCTRL_IRQ);
        let usb_driver = Driver::new(p.USB, irq);

        usb::init(&spawner, usb_driver);
    } else {
        log::info!("No usb connected");
    }

    logger::setup_logger();
    messages::init(&spawner);
    #[cfg(feature = "bootloader")]
    fw_update::init(&spawner, p.WATCHDOG, p.FLASH);

    let (_pio0, sm0, sm1, _sm2, _sm3) = p.PIO0.split();
    let usart_pin = p.PIN_1.into();
    // let usart_pin = p.PIN_25.into();

    interboard::init(&spawner, sm0, sm1, usart_pin);

    spawner.must_spawn(blinky(Output::new(p.PIN_25, embassy_rp::gpio::Level::Low)));

    let mut counter = 0u8;
    loop {
        counter = counter.wrapping_add(1);

        Timer::after(Duration::from_secs(1)).await;

        if side::get_side().is_left() {
            interboard::send_msg(reliable_msg(shared::device_to_device::DeviceToDevice::Ping)).await;
        }
    }
}
