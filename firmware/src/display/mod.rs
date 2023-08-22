use alloc::boxed::Box;
use embassy_rp::{
    multicore::{spawn_core1, Stack},
    peripherals::{CORE1, PIN_11, PIN_12, PIN_13, PIN_22, PIN_23, SPI0},
    spi::Spi,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use slint::platform::software_renderer::Rgb565Pixel;

use self::{backend::PicoBackend, draw_buffer::DrawBuffer};

mod backend;
mod display_interface;
mod draw_buffer;

slint::include_modules!();

const DISPLAY_SIZE: slint::PhysicalSize = slint::PhysicalSize::new(240, 240);
pub type TargetPixel = Rgb565Pixel;

fn run(spi: SPI0, clk: PIN_22, mosi: PIN_23, cs: PIN_12, dc: PIN_11, rst: PIN_13) -> ! {
    let mut config = embassy_rp::spi::Config::default();
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;
    config.frequency = 62_500_000;

    let dc = embassy_rp::gpio::Output::new(dc, embassy_rp::gpio::Level::Low);
    let cs = embassy_rp::gpio::Output::new(cs, embassy_rp::gpio::Level::Low);
    let rst = embassy_rp::gpio::Output::new(rst, embassy_rp::gpio::Level::Low);

    let spi = Spi::new_blocking_txonly(spi, clk, mosi, config);
    let spi = ExclusiveDevice::new(spi, cs, embassy_time::Delay);

    let di = display_interface::SPIInterfaceNoCS::new(spi, dc);

    let display = mipidsi::Builder::st7789(di)
        .init(&mut embassy_time::Delay, Some(rst))
        .unwrap();

    let buffer_provider = DrawBuffer {
        display,
        buffer: alloc::vec![Rgb565Pixel::default(); DISPLAY_SIZE.width as _].leak(),
    };

    slint::platform::set_platform(Box::new(PicoBackend::new(buffer_provider))).unwrap();

    let window = MainWindow::new().unwrap();

    window.run().unwrap();

    loop {}
}

static mut CORE1_STACK: Stack<4096> = Stack::new();

pub fn init(
    core1: CORE1,
    spi: SPI0,
    clk: PIN_22,
    mosi: PIN_23,
    cs: PIN_12,
    dc: PIN_11,
    rst: PIN_13,
) {
    spawn_core1(core1, unsafe { &mut CORE1_STACK }, move || {
        run(spi, clk, mosi, cs, dc, rst)
    });
}
