#![no_std]
#![allow(incomplete_features)]
#![feature(
    type_alias_impl_trait,
    trait_alias,
    async_fn_in_trait,
    maybe_uninit_uninit_array,
    const_maybe_uninit_uninit_array,
    maybe_uninit_array_assume_init,
    const_maybe_uninit_array_assume_init,
    const_mut_refs,
    const_maybe_uninit_write
)]

use atomic_polyfill::AtomicU32;
use embassy_executor::{Executor, Spawner};
use embassy_rp::bind_interrupts;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{AnyPin, Input, Pin};
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::peripherals::{PIN_19, PIN_29, USB};
use embassy_rp::pio::Pio;
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_rp::usb::Driver;
use embassy_time::{Duration, Timer};
use shared::side::KeyboardSide;

#[cfg(not(feature = "probe"))]
use panic_reset as _;
#[cfg(feature = "probe")]
use {defmt_rtt as _, panic_probe as _};

use utils::{log, singleton};

use crate::keys::ScannerInstance;

pub mod event;
#[cfg(feature = "bootloader")]
pub mod fw_update;
pub mod interboard;
pub mod keys;
pub mod logger;
pub mod messages;
pub mod rgb;
pub mod side;
pub mod trackpad;
pub mod usb;
pub mod utils;

pub static VERSION: &str = "0.1.0";

fn detect_usb(pin: Input<'_, PIN_19>) -> bool {
    let connected = pin.is_high();
    log::info!("Usb connected? {}", connected);
    connected
}

fn detect_side(pin: Input<'_, PIN_29>) -> KeyboardSide {
    let is_right = pin.is_high();
    let side = if is_right {
        KeyboardSide::Right
    } else {
        KeyboardSide::Left
    };
    log::info!("I'm the {:?} side", side);
    side
}

#[embassy_executor::task]
async fn blinky(mut pin: Output<'static, AnyPin>) {
    loop {
        pin.set_high();
        Timer::after(Duration::from_secs(1)).await;

        pin.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[link_section = ".uninit.bootloader_magic"]
#[used]
static BOOTLOADER_MAGIC: AtomicU32 = AtomicU32::new(0);

const MAGIC_TOKEN: u32 = 0xCAFEB0BA;

unsafe fn check_bootloader() {
    const CYCLES_PER_US: usize = 125;
    const WAIT_CYCLES: usize = 100 * 1000 * CYCLES_PER_US;

    if BOOTLOADER_MAGIC.load(atomic_polyfill::Ordering::SeqCst) != MAGIC_TOKEN {
        BOOTLOADER_MAGIC.store(MAGIC_TOKEN, atomic_polyfill::Ordering::SeqCst);

        cortex_m::asm::delay(WAIT_CYCLES as u32);
        BOOTLOADER_MAGIC.store(0, atomic_polyfill::Ordering::SeqCst);
        return;
    }

    BOOTLOADER_MAGIC.store(0, atomic_polyfill::Ordering::SeqCst);

    reset_to_usb_boot(1 << 17, 0);
}

pub async fn main(spawner: &Spawner) {
    let p = embassy_rp::init(Default::default());
    unsafe {
        check_bootloader();
    }

    log::info!("Just a whisper, I hear it in my ghost.");

    // not sure if this makes the usb detection happier
    Timer::after(Duration::from_micros(100)).await;

    let s = detect_side(Input::new(p.PIN_29, embassy_rp::gpio::Pull::Down));
    side::init(
        s,
        detect_usb(Input::new(p.PIN_19, embassy_rp::gpio::Pull::Down)),
    );

    if side::this_side_has_usb() {
        log::info!("usb connected");
        bind_interrupts!(struct Irqs {
            USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
        });

        let usb_driver = Driver::new(p.USB, Irqs);

        usb::init(&spawner, usb_driver);
    } else {
        log::info!("No usb connected");
    }

    logger::init();
    messages::init(&spawner);
    #[cfg(feature = "bootloader")]
    fw_update::init(&spawner, p.WATCHDOG, p.FLASH);

    let mut pio0 = Pio::new(p.PIO0);
    interboard::init(&spawner, &mut pio0.common, pio0.sm0, pio0.sm1, p.PIN_1);
    let mut pio1 = Pio::new(p.PIO1);
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
    }

    spawner.must_spawn(blinky(Output::new(
        p.PIN_17.degrade(),
        embassy_rp::gpio::Level::Low,
    )));

    let mut counter = 0u8;
    loop {
        counter = counter.wrapping_add(1);

        Timer::after(Duration::from_secs(1)).await;

        log::info!("tick");
    }
}
