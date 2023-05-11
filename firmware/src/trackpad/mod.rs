use embassy_executor::Spawner;
use embassy_rp::{
    dma::AnyChannel,
    gpio::{self, Output},
    peripherals::{PIN_20, PIN_21, PIN_22, PIN_23, SPI0},
    spi::{self, Async, Spi},
};
use embassy_time::Duration;
use embedded_hal_async::spi::ExclusiveDevice;

pub mod driver;
mod glide;
pub mod regs;

type TrackpadSpi = ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static, PIN_21>>;

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
    let config = spi::Config::default();
    let spi = Spi::new(spi, clk, mosi, miso, tx_dma, rx_dma, config);
    let spi = ExclusiveDevice::new(spi, Output::new(cs, gpio::Level::High));

    spawner.must_spawn(trackpad_task(spi));
}

#[embassy_executor::task]
async fn trackpad_task(spi: TrackpadSpi) {
    let mut trackpad = driver::Trackpad::<_, 35>::new(
        spi,
        driver::PositionMode::Relative,
        driver::Overlay::Curved,
        None,
    );

    trackpad.init().await;

    let mut ticker = embassy_time::Ticker::every(Duration::from_millis(10));

    loop {
        if let Some(report) = trackpad.get_report().await {
            crate::log::info!("trackpad report: {:?}", report);
        }

        ticker.next().await;
    }
}
