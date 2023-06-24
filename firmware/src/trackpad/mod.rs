use atomic_polyfill::AtomicBool;
use embassy_executor::Spawner;
use embassy_rp::{
    dma::AnyChannel,
    gpio::{self, Output},
    peripherals::{PIN_20, PIN_21, PIN_22, PIN_23, SPI0},
    spi::{self, Async, Spi},
};
use embassy_time::Duration;
use embedded_hal_async::spi::ExclusiveDevice;
use shared::hid::MouseReport;

use crate::{keys::KEY_EVENTS, utils::Ticker};

pub mod driver;
mod glide;
pub mod regs;

type TrackpadSpi = ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static, PIN_21>>;

static SCROLLING: AtomicBool = AtomicBool::new(false);

#[allow(clippy::too_many_arguments)]
pub fn init(
    spawner: &Spawner,
    spi: SPI0,
    clk: PIN_22,
    mosi: PIN_23,
    miso: PIN_20,
    cs: PIN_21,
    tx_dma: AnyChannel,
    rx_dma: AnyChannel,
) {
    let mut config = spi::Config::default();
    config.phase = spi::Phase::CaptureOnSecondTransition;
    let spi = Spi::new(spi, clk, mosi, miso, tx_dma, rx_dma, config);
    let spi = ExclusiveDevice::new(spi, Output::new(cs, gpio::Level::Low));

    spawner.must_spawn(trackpad_task(spi));
    spawner.must_spawn(ad_hoc_key_handler())
}

#[embassy_executor::task]
async fn trackpad_task(spi: TrackpadSpi) {
    let mut trackpad = driver::Trackpad::<_, 35>::new(
        spi,
        driver::PositionMode::Absolute,
        driver::Overlay::Curved,
        driver::TransformMode::Rotate90,
        None,
    );

    if let Err(_e) = trackpad.init().await {
        crate::log::error!("Couldn't init trackpad");
        return;
    }

    let mut ticker = Ticker::every(Duration::from_millis(10));

    loop {
        match trackpad.get_report().await {
            Ok(Some(report)) => {
                let rep = if SCROLLING.load(atomic_polyfill::Ordering::Relaxed) {
                    MouseReport {
                        pan: report.0,
                        wheel: report.1,
                        x: 0,
                        y: 0,
                    }
                } else {
                    MouseReport {
                        x: report.0,
                        y: report.1,
                        wheel: 0,
                        pan: 0,
                    }
                };
                crate::usb::hid::send_mouse_hid_to_host(rep).await;
                // crate::log::info!("trackpad report: {:?}", report);
            }
            Err(_e) => {
                crate::log::error!("Failed to get a trackpad report");
            }
            _ => (),
        }

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn ad_hoc_key_handler() {
    let mut sub = KEY_EVENTS.subscriber().unwrap();

    loop {
        let evt = sub.next_message_pure().await;

        match evt.coord() {
            (3, 9) => {
                SCROLLING.store(evt.is_press(), atomic_polyfill::Ordering::SeqCst);
            }
            _ => {}
        }
    }
}
