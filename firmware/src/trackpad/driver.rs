use embassy_time::{with_timeout, Duration, Timer};
use embedded_hal_async::spi::SpiDevice;

use super::regs::{self, Register};

pub struct Trackpad<SPI> {
    spi: SPI,
}

const WRITE_MASK: u8 = 0x80;
const READ_MASK: u8 = 0xA0;
const FILLER_BYTE: u8 = 0xFC;

/// utility stuff
impl<SPI: SpiDevice> Trackpad<SPI> {
    async fn set_feed_enable(&mut self, enabled: bool) {
        let mut feed_config = self.rap_read_reg::<regs::FeedConfig>().await;
        feed_config.set_feed_enable(enabled);
        self.rap_write_reg(feed_config).await;
    }

    async fn clear_flags(&mut self) {
        self.rap_write_reg(
            regs::Status::def()
                .with_command_complete(false)
                .with_data_ready(false),
        )
        .await;
        Timer::after(Duration::from_micros(50)).await;
    }
}

/// era reading
impl<SPI: SpiDevice> Trackpad<SPI> {
    async fn era_read(&mut self, address: u16, buf: &mut [u8]) {
        self.set_feed_enable(false).await;

        let [upper, lower] = address.to_be_bytes();
        self.rap_write_byte(regs::HOSTREG__EXT_REG_AXS_ADDR_HIGH, upper)
            .await;
        self.rap_write_byte(regs::HOSTREG__EXT_REG_AXS_ADDR_LOW, lower)
            .await;

        for dst in buf {
            self.rap_write_byte(
                regs::HOSTREG__EXT_REG_AXS_CTRL,
                regs::HOSTREG__EREG_AXS__INC_ADDR_READ | regs::HOSTREG__EREG_AXS__READ,
            )
            .await;

            let _ = with_timeout(Duration::from_millis(20), async {
                loop {
                    let v = self.rap_read_byte(regs::HOSTREG__EXT_REG_AXS_CTRL).await;
                    if v == 0 {
                        break;
                    }
                }
            })
            .await;

            *dst = self.rap_read_byte(regs::HOSTREG__EXT_REG_AXS_VALUE).await;
        }
    }
}

/// rap reading
impl<SPI: SpiDevice> Trackpad<SPI> {
    async fn rap_read_reg<R: regs::Register>(&mut self) -> R {
        let mut b: u8 = 0u8;
        self.rap_read(R::REG, core::slice::from_mut(&mut b)).await;
        R::from_byte(b)
    }

    async fn rap_write_reg<R: regs::Register>(&mut self, value: R) {
        self.rap_write(R::REG, &[value.to_byte()]).await;
    }

    async fn rap_read_byte(&mut self, address: u8) -> u8 {
        let mut b: u8 = 0u8;
        self.rap_read(address, core::slice::from_mut(&mut b)).await;
        b
    }

    async fn rap_write_byte(&mut self, address: u8, value: u8) {
        self.rap_write(address, &[value]).await;
    }

    async fn rap_read(&mut self, address: u8, buf: &mut [u8]) {
        let cmd = address | READ_MASK;
        let _ = self.spi.write(&[cmd, FILLER_BYTE, FILLER_BYTE]).await;
        for dst in buf {
            let _ = self
                .spi
                .transfer(core::slice::from_mut(dst), &[FILLER_BYTE])
                .await;
        }
    }

    async fn rap_write(&mut self, address: u8, buf: &[u8]) {
        let cmd = address | WRITE_MASK;
        let _ = self.spi.write_transaction(&[&[cmd], buf]).await;
    }
}
