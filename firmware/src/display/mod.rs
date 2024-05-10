use core::sync::atomic::{AtomicBool, AtomicUsize};

use alloc::{boxed::Box, rc::Rc};
use embassy_executor::Spawner;
use embassy_rp::{
    multicore::{spawn_core1, Stack},
    peripherals::{CORE1, PIN_11, PIN_12, PIN_13, PIN_22, PIN_23, PWM_SLICE6, SPI0},
    spi::Spi,
};
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use slint::platform::software_renderer::Rgb565Pixel;

use crate::metrics::{self, Metrics, METRIC_UPDATES};

use self::{backend::PicoBackend, draw_buffer::DrawBuffer};

mod backend;
mod display_interface;
mod draw_buffer;

slint::include_modules!();

const DISPLAY_SIZE: slint::PhysicalSize = slint::PhysicalSize::new(240, 240);
pub type TargetPixel = Rgb565Pixel;

static DISPLAY_OFF: AtomicBool = AtomicBool::new(false);

fn run(spi: SPI0, clk: PIN_22, mosi: PIN_23, cs: PIN_12, dc: PIN_11) -> ! {
    let mut config = embassy_rp::spi::Config::default();
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;
    config.frequency = 62_500_000;

    let dc = embassy_rp::gpio::Output::new(dc, embassy_rp::gpio::Level::Low);
    let cs = embassy_rp::gpio::Output::new(cs, embassy_rp::gpio::Level::Low);

    let spi = Spi::new_blocking_txonly(spi, clk, mosi, config);
    let spi = ExclusiveDevice::new(spi, cs, embassy_time::Delay).unwrap();

    let di = display_interface::SPIInterfaceNoCS::new(spi, dc);

    let display = mipidsi::Builder::new(mipidsi::models::ST7789, di)
        .display_size(DISPLAY_SIZE.width as _, DISPLAY_SIZE.height as _)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .init(&mut embassy_time::Delay)
        .unwrap();

    let buffer_provider = DrawBuffer {
        display,
        buffer: alloc::vec![Rgb565Pixel::default(); DISPLAY_SIZE.width as _].leak(),
    };

    slint::platform::set_platform(Box::new(PicoBackend::new(&DISPLAY_OFF, buffer_provider)))
        .unwrap();

    let window = Rc::new(MainWindow::new().unwrap());

    let update_timer = slint::Timer::default();
    update_timer.start(
        slint::TimerMode::Repeated,
        core::time::Duration::from_millis(16),
        {
            let window = Rc::clone(&window);
            move || {
                window.set_keypresses(KEYS_PRESSED.load(portable_atomic::Ordering::Relaxed) as i32);
                window.set_ticks(
                    crate::utils::executor_metrics::WAKEUPS.load(portable_atomic::Ordering::Relaxed)
                        as i32,
                );

                let awake =
                    crate::utils::executor_metrics::AWAKE.load(portable_atomic::Ordering::Relaxed);
                let sleep =
                    crate::utils::executor_metrics::SLEEP.load(portable_atomic::Ordering::Relaxed);

                let percentage_awake = 100 - ((100 * sleep) / (sleep + awake + 1));

                window.set_cpu_util(percentage_awake as i32);
            }
        },
    );

    window.run().unwrap();

    loop {}
}

static KEYS_PRESSED: AtomicUsize = AtomicUsize::new(0);

#[embassy_executor::task]
async fn metrics_updater(bl: PIN_13, pwm: PWM_SLICE6) {
    let mut sub = METRIC_UPDATES.subscriber().unwrap();
    let mut pwm_cfg = embassy_rp::pwm::Config::default();
    pwm_cfg.top = 256;
    pwm_cfg.compare_b = 256;
    let mut bl = embassy_rp::pwm::Pwm::new_output_b(pwm, bl, pwm_cfg.clone());

    metrics::request_sync().await;

    loop {
        let Metrics { keys_pressed } = match embassy_time::with_timeout(
            Duration::from_secs(30),
            sub.next_message_pure(),
        )
        .await
        {
            Ok(m) => m,
            Err(_e) => {
                for n in (0..=256).rev() {
                    pwm_cfg.compare_b = n;
                    bl.set_config(&pwm_cfg);
                    Timer::after(Duration::from_hz(256)).await;
                }
                DISPLAY_OFF.store(true, portable_atomic::Ordering::Relaxed);

                let r = sub.next_message_pure().await;

                DISPLAY_OFF.store(false, portable_atomic::Ordering::Relaxed);

                for n in 0..=256 {
                    pwm_cfg.compare_b = n;
                    bl.set_config(&pwm_cfg);
                    Timer::after(Duration::from_hz(256)).await;
                }

                r
            }
        };

        KEYS_PRESSED.store(keys_pressed.0, portable_atomic::Ordering::Release);
    }
}

static mut CORE1_STACK: Stack<16384> = Stack::new();

pub fn init(
    spawner: &Spawner,
    core1: CORE1,
    spi: SPI0,
    clk: PIN_22,
    mosi: PIN_23,
    cs: PIN_12,
    dc: PIN_11,
    bl: PIN_13,
    pwm: PWM_SLICE6,
) {
    spawner.must_spawn(metrics_updater(bl, pwm));

    spawn_core1(
        core1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || run(spi, clk, mosi, cs, dc),
    );
}
