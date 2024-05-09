#![no_std]
#![allow(incomplete_features, async_fn_in_trait)]
#![feature(
    iter_repeat_n,
    type_alias_impl_trait,
    impl_trait_in_assoc_type,
    trait_alias,
    maybe_uninit_uninit_array,
    const_maybe_uninit_uninit_array,
    maybe_uninit_array_assume_init,
    const_maybe_uninit_array_assume_init,
    const_mut_refs,
    const_maybe_uninit_write,
    option_take_if,
    // generic_const_exprs: would be nice but breaks things
)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::mem::ManuallyDrop;

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_17, USB};
use embassy_rp::pio::Pio;
use embassy_rp::usb::Driver;
use embassy_time::{Duration, Timer};
use shared::side::KeyboardSide;

#[cfg(not(feature = "probe"))]
use panic_reset as _;
#[cfg(feature = "probe")]
use {defmt_rtt as _, panic_probe as _};

use utils::log;

use crate::keys::ScannerInstance;

#[cfg(feature = "alloc")]
mod allocator;
#[cfg(feature = "binaryinfo")]
pub mod binary_info;
#[cfg(feature = "display-slint")]
mod display;
pub mod event;
mod flash;
pub mod interboard;
pub mod keys;
pub mod logger;
pub mod messages;
mod metrics;
pub mod rgb;
pub mod rng;
pub mod side;
pub mod trackpad;
pub mod usb;
pub mod utils;

pub fn set_status_led(value: Level) {
    unsafe { ManuallyDrop::new(Output::new(PIN_17::steal(), value)).set_level(value) };
}

pub static VERSION: &str = "0.1.0";

fn detect_usb(pin: Input) -> bool {
    let connected = pin.is_high();
    log::info!("Usb connected? {}", connected);
    connected
}

fn detect_side(pin: Input) -> KeyboardSide {
    let is_right = pin.is_high();
    let side = if is_right {
        KeyboardSide::Right
    } else {
        KeyboardSide::Left
    };
    log::info!("I'm the {:?} side", side);
    side
}

bind_interrupts!(struct PioIrq0 {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
});

bind_interrupts!(struct PioIrq1 {
    PIO1_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO1>;
});

bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

pub async fn main(spawner: Spawner) {
    let mut config = embassy_rp::config::Config::default();
    if let Some(xosc) = config.clocks.xosc.as_mut() {
        xosc.sys_pll = Some(embassy_rp::clocks::PllConfig {
            refdiv: 1,
            fbdiv: 125,
            post_div1: 3,
            post_div2: 2,
        });
    }
    let p = embassy_rp::init(config);

    set_status_led(Level::High);

    // not sure if this makes the usb detection happier
    Timer::after(Duration::from_millis(100)).await;

    #[cfg(feature = "alloc")]
    allocator::init();

    log::info!("Just a whisper, I hear it in my ghost.");

    set_status_led(Level::High);

    let s = detect_side(Input::new(p.PIN_29, embassy_rp::gpio::Pull::Down));
    side::init(
        s,
        detect_usb(Input::new(p.PIN_19, embassy_rp::gpio::Pull::Down)),
    );

    if side::this_side_has_usb() {
        log::info!("usb connected");
        let usb_driver = Driver::new(p.USB, UsbIrqs);

        usb::init(&spawner, usb_driver);
    } else {
        log::info!("No usb connected");
    }

    messages::init(&spawner);

    #[cfg(not(feature = "probe"))]
    logger::init();

    rng::init();

    let mut pio0 = Pio::new(p.PIO0, PioIrq0);
    interboard::init(&spawner, &mut pio0.common, pio0.sm0, pio0.sm1, p.PIN_1);

    flash::init(p.FLASH, p.DMA_CH3.degrade()).await;

    let mut pio1 = Pio::new(p.PIO1, PioIrq1);
    rgb::init(&spawner, &mut pio1.common, pio1.sm0, p.PIN_10, p.DMA_CH2);

    let scanner = ScannerInstance::new(
        (
            Input::new(p.PIN_4, Pull::Up),
            Input::new(p.PIN_5, Pull::Up),
            Input::new(p.PIN_27, Pull::Up),
            Input::new(p.PIN_26, Pull::Up),
        ),
        (
            Output::new(p.PIN_8, Level::Low),
            Output::new(p.PIN_9, Level::Low),
            Output::new(p.PIN_7, Level::Low),
            Output::new(p.PIN_6, Level::Low),
            Output::new(p.PIN_28, Level::Low),
        ),
    );

    keys::init(&spawner, scanner);

    if side::get_side().is_right() {
        log::info!("Initializing trackpad");
        trackpad::init(
            &spawner,
            p.SPI0,
            p.PIN_22,
            p.PIN_23,
            p.PIN_20,
            p.PIN_21,
            p.DMA_CH0.degrade(),
            p.DMA_CH1.degrade(),
        );
    } else {
        #[cfg(feature = "display-slint")]
        {
            display::init(
                &spawner,
                p.CORE1,
                p.SPI0,
                p.PIN_22,
                p.PIN_23,
                p.PIN_12,
                p.PIN_11,
                p.PIN_13,
                p.PWM_SLICE6,
            );
        }
    }

    metrics::init(&spawner).await;

    log::info!("All set up, have fun :)");

    // allowing the main task to exit somehow causes the LED task to break?
    //
    // everything else still works though which is pretty weird
    //
    // anyway, just pend the task forever so it isn't dropped
    core::future::pending::<()>().await;
}
