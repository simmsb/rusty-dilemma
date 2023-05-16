use embassy_time::{with_timeout, Duration, Timer};
use embedded_hal_async::spi::SpiDevice;

use super::{
    glide::{GlideConfig, GlideContext},
    regs::{self, Register},
};

pub struct Trackpad<SPI, const DIAMETER: u32> {
    spi: SPI,
    position_mode: PositionMode,
    overlay: Overlay,
    transform: TransformMode,
    relative_remainder: (i16, i16),
    glide: Option<GlideContext>,
    last_pos: Option<(u16, u16)>,
    scale: u16,
    last_scale: u16,
}

pub enum TransformMode {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl TransformMode {
    fn transform(&self, x: i8, y: i8) -> (i8, i8) {
        match self {
            TransformMode::Normal => (x, y),
            TransformMode::Rotate90 => (y, -x),
            TransformMode::Rotate180 => (-x, -y),
            TransformMode::Rotate270 => (-y, x),
        }
    }
}

pub enum Overlay {
    Curved,
    Other,
}

pub enum PositionMode {
    Absolute,
    Relative,
}

#[derive(Debug)]
#[cfg_attr(feature = "probe", derive(defmt::Format))]
pub enum Reading {
    Absolute {
        x: u16,
        y: u16,
        z: u16,
        buttons: u8,
        touch_down: bool,
    },
    Relative {
        dx: i16,
        dy: i16,
        wheel_count: i8,
        buttons: u8,
    },
}

impl Reading {
    const ABS_X_MIN: u16 = 127;
    const ABS_X_MAX: u16 = 1919;
    const ABS_X_RANGE: u16 = Self::ABS_X_MAX - Self::ABS_X_MIN;

    const ABS_Y_MIN: u16 = 63;
    const ABS_Y_MAX: u16 = 1471;
    const ABS_Y_RANGE: u16 = Self::ABS_Y_MAX - Self::ABS_Y_MIN;

    const REL_X_RANGE: u16 = 256;
    const REL_Y_RANGE: u16 = 256;

    fn resolve_abs(x: u16, y: u16) -> (u16, u16) {
        let x = x.clamp(Self::ABS_X_MIN, Self::ABS_X_MAX) - Self::ABS_X_MIN;
        let y = y.clamp(Self::ABS_Y_MIN, Self::ABS_Y_MAX) - Self::ABS_Y_MIN;

        (x, y)
    }
}

const WRITE_MASK: u8 = 0x80;
const READ_MASK: u8 = 0xA0;
const FILLER_BYTE: u8 = 0xFC;

fn saturating_i16_to_i8(v: i16) -> i8 {
    v.clamp(i8::MIN as i16, i8::MAX as i16) as i8
}

impl<SPI: SpiDevice, const DIAMETER: u32> Trackpad<SPI, DIAMETER> {
    pub fn new(
        spi: SPI,
        position_mode: PositionMode,
        overlay: Overlay,
        transform: TransformMode,
        glide_config: Option<GlideConfig>,
    ) -> Self {
        Self {
            spi,
            position_mode,
            overlay,
            transform,
            glide: glide_config.map(|c| GlideContext::new(c)),
            relative_remainder: (0, 0),
            last_pos: None,
            scale: ((800 as u32 * DIAMETER * 10) / 254) as u16,
            last_scale: 0,
        }
    }

    pub fn set_scale(&mut self, cpi: u16) {
        self.scale = ((cpi as u32 * DIAMETER * 10) / 254) as u16;
    }

    pub async fn init(&mut self) -> Result<(), SPI::Error> {
        self.rap_write_reg(regs::SystemConfig::def().with_reset(true))
            .await?;

        Timer::after(Duration::from_millis(30)).await;

        self.rap_write_reg(regs::SystemConfig::def()).await?;

        Timer::after(Duration::from_micros(50)).await;

        self.clear_flags().await?;

        match self.position_mode {
            PositionMode::Absolute => {
                self.rap_write_reg(regs::FeedConfig2::def()).await?;
                self.rap_write_reg(regs::FeedConfig1::def().with_data_type_relo0_abs1(true))
                    .await?;
                self.rap_write_reg(regs::ZIdle(5)).await?;
            }
            PositionMode::Relative => {
                let cfg = regs::FeedConfig2::new()
                    .with_glide_extend_disable(true)
                    .with_intellimouse_mode(true)
                    .with_all_tap_disable(true)
                    .with_secondary_tap_disable(true)
                    .with_scroll_disable(true);

                self.rap_write_reg(cfg).await?;
                self.rap_write_reg(regs::FeedConfig1::def()).await?;
            }
        }

        let should_calibrate = match self.overlay {
            Overlay::Curved => {
                self.set_adc_attenuation(regs::AdcAttenuation::X2).await?;
                self.tune_edge_sensivity().await?;
                true
            }
            Overlay::Other => self.set_adc_attenuation(regs::AdcAttenuation::X2).await?,
        };

        if should_calibrate {
            self.calibrate().await?;
        }

        self.set_feed_enable(true).await?;

        Ok(())
    }

    pub async fn get_report(&mut self) -> Result<Option<(i8, i8)>, SPI::Error> {
        let reading = self.read_data().await?;
        // crate::log::info!("raw reading: {:?}", reading);

        let glide_report = self.glide.as_mut().and_then(|g| g.check());

        let Some(reading) = reading else { return Ok(None); };

        let reading = self.scale_reading(reading);

        let (mut report_x, mut report_y) = (0, 0);

        match reading {
            Reading::Absolute {
                x,
                y,
                z,
                buttons: _,
                touch_down,
            } => {
                if !touch_down {
                    self.last_pos = None;
                }

                // crate::log::info!("handling report: {:?} last: {:?}", reading, self.last_pos);

                if self.last_scale != 0 && self.last_scale == self.scale && x != 0 && y != 0 {
                    if let Some((last_x, last_y)) = self.last_pos {
                        report_x = saturating_i16_to_i8(x as i16 - last_x as i16);
                        report_y = saturating_i16_to_i8(y as i16 - last_y as i16);
                    }
                }

                if touch_down {
                    self.last_pos = Some((x, y));
                    self.last_scale = self.scale;
                }

                if let Some(glide_ctx) = &mut self.glide {
                    if touch_down {
                        glide_ctx.update(report_x as i16, report_y as i16, z)
                    }

                    if glide_report.is_none() {
                        if let Some(report) = glide_ctx.start() {
                            report_x = report.dx;
                            report_y = report.dy;
                        }
                    }
                }
            }
            Reading::Relative {
                dx,
                dy,
                wheel_count: _,
                buttons: _,
            } => {
                report_x = saturating_i16_to_i8(dx);
                report_y = saturating_i16_to_i8(dy);
            }
        }

        Ok(Some(self.transform.transform(report_x, report_y)))
    }

    async fn read_data(&mut self) -> Result<Option<Reading>, SPI::Error> {
        let status = self.rap_read_reg::<regs::Status>().await?;
        if !status.data_ready() {
            return Ok(None);
        }

        // crate::log::info!("status: {:?}", status);

        let mut data = [0u8; 6];
        self.rap_read(regs::Packet0::REG, &mut data).await?;
        self.clear_flags().await?;

        // crate::log::info!("read raw bytes: {:?}", data);

        match self.position_mode {
            PositionMode::Absolute => {
                let buttons = data[0] & 0x3f;
                let x = (data[2] as u16) | ((data[4] & 0x0F) as u16) << 8;
                let y = (data[3] as u16) | ((data[4] & 0xF0) as u16) << 4;
                let z = (data[5] & 0x3f) as u16;
                let touch_down = x != 0 || y != 0;

                let reading = Reading::Absolute {
                    x,
                    y,
                    z,
                    buttons,
                    touch_down,
                };
                Ok(Some(reading))
            }
            PositionMode::Relative => {
                let buttons = data[0] & 0x07;

                let dx = if (data[0] & 0x10) != 0 && data[1] != 0 {
                    -(256i16 - data[1] as i16)
                } else {
                    data[1] as i16
                };

                let dy = if (data[0] & 0x20) != 0 && data[2] != 0 {
                    256i16 - data[2] as i16
                } else {
                    -(data[2] as i16)
                };

                let wheel_count = i8::from_be_bytes([data[2]]);

                Ok(Some(Reading::Relative {
                    dx,
                    dy,
                    wheel_count,
                    buttons,
                }))
            }
        }
    }

    fn scale_reading(&mut self, reading: Reading) -> Reading {
        match reading {
            Reading::Absolute {
                x,
                y,
                z,
                buttons,
                touch_down,
            } => {
                let (x, y) = Reading::resolve_abs(x, y);

                let x = (x as u32 * self.scale as u32 / Reading::ABS_X_RANGE as u32) as u16;
                let y = (y as u32 * self.scale as u32 / Reading::ABS_Y_RANGE as u32) as u16;

                Reading::Absolute {
                    x,
                    y,
                    z,
                    buttons,
                    touch_down,
                }
            }
            Reading::Relative {
                dx,
                dy,
                wheel_count,
                buttons,
            } => {
                let (dx, dx_r) = num::integer::div_rem(
                    dx as i32 * self.scale as i32 + self.relative_remainder.0 as i32,
                    Reading::REL_X_RANGE as i32,
                );
                let (dy, dy_r) = num::integer::div_rem(
                    dy as i32 * self.scale as i32 + self.relative_remainder.1 as i32,
                    Reading::REL_Y_RANGE as i32,
                );

                self.relative_remainder = (dx_r as i16, dy_r as i16);

                Reading::Relative {
                    dx: dx as i16,
                    dy: dy as i16,
                    wheel_count,
                    buttons,
                }
            }
        }
    }
}

/// utility stuff
impl<SPI: SpiDevice, const DIAMETER: u32> Trackpad<SPI, DIAMETER> {
    async fn set_feed_enable(&mut self, enabled: bool) -> Result<(), SPI::Error> {
        let mut feed_config = self.rap_read_reg::<regs::FeedConfig1>().await?;
        feed_config.set_feed_enable(enabled);
        self.rap_write_reg(feed_config).await?;
        Ok(())
    }

    async fn clear_flags(&mut self) -> Result<(), SPI::Error> {
        self.rap_write_reg(
            regs::Status::def()
                .with_command_complete(false)
                .with_data_ready(false),
        )
        .await?;
        Timer::after(Duration::from_micros(50)).await;
        Ok(())
    }

    async fn set_adc_attenuation(
        &mut self,
        gain: regs::AdcAttenuation,
    ) -> Result<bool, SPI::Error> {
        let mut cfg = self.era_read_reg::<regs::TrackAdcConfig>().await?;

        if gain == cfg.attenuate() {
            return Ok(false);
        }

        cfg.set_attenuate(gain);
        self.era_write_reg(cfg).await?;
        self.era_read_reg::<regs::TrackAdcConfig>().await?;

        return Ok(true);
    }

    async fn tune_edge_sensivity(&mut self) -> Result<(), SPI::Error> {
        self.era_read_reg::<regs::XAxisWideZMin>().await?;
        self.era_write_reg(regs::XAxisWideZMin(0x04)).await?;
        self.era_read_reg::<regs::XAxisWideZMin>().await?;

        self.era_read_reg::<regs::YAxisWideZMin>().await?;
        self.era_write_reg(regs::YAxisWideZMin(0x03)).await?;
        self.era_read_reg::<regs::YAxisWideZMin>().await?;

        Ok(())
    }

    async fn calibrate(&mut self) -> Result<(), SPI::Error> {
        let cfg = self.rap_read_reg::<regs::CalConfig>().await?;
        self.rap_write_reg(cfg.with_calibrate(true)).await?;

        let _ = with_timeout(Duration::from_millis(200), async {
            loop {
                let Ok(v) = self.rap_read_reg::<regs::CalConfig>().await else { continue; };
                if !v.calibrate() {
                    break;
                }
            }
        })
        .await;

        self.clear_flags().await?;

        Ok(())
    }

    #[allow(unused)]
    async fn set_cursor_smoothing(&mut self, enabled: bool) -> Result<(), SPI::Error> {
        let cfg = self.rap_read_reg::<regs::FeedConfig3>().await?;
        self.rap_write_reg(cfg.with_disable_cross_rate_smoothing(!enabled))
            .await
    }
}

/// era reading
impl<SPI: SpiDevice, const DIAMETER: u32> Trackpad<SPI, DIAMETER> {
    async fn era_read_reg<R: regs::Register<u16>>(&mut self) -> Result<R, SPI::Error> {
        let mut b: u8 = 0u8;
        self.era_read(R::REG, core::slice::from_mut(&mut b)).await?;
        Ok(R::from_byte(b))
    }

    async fn era_write_reg<R: regs::Register<u16>>(&mut self, value: R) -> Result<(), SPI::Error> {
        self.era_write(R::REG, value.to_byte()).await
    }

    async fn era_read(&mut self, address: u16, buf: &mut [u8]) -> Result<(), SPI::Error> {
        self.set_feed_enable(false).await?;

        let [upper, lower] = address.to_be_bytes();
        self.rap_write_reg(regs::AXSAddrHigh(upper)).await?;
        self.rap_write_reg(regs::AXSAddrLow(lower)).await?;

        for dst in buf {
            self.rap_write_reg(
                regs::AXSCtrl::def()
                    .with_inc_addr_read(true)
                    .with_read(true),
            )
            .await?;

            let _ = with_timeout(Duration::from_millis(20), async {
                loop {
                    let Ok(v) = self.rap_read_reg::<regs::AXSCtrl>().await else { continue; };
                    if u8::from(v) == 0 {
                        break;
                    }
                }
            })
            .await;

            *dst = self.rap_read_reg::<regs::AXSValue>().await?.0;
        }

        self.clear_flags().await?;

        Ok(())
    }

    async fn era_write(&mut self, address: u16, data: u8) -> Result<(), SPI::Error> {
        self.set_feed_enable(false).await?;

        self.rap_write_reg(regs::AXSValue(data)).await?;

        let [upper, lower] = address.to_be_bytes();
        self.rap_write_reg(regs::AXSAddrHigh(upper)).await?;
        self.rap_write_reg(regs::AXSAddrLow(lower)).await?;

        self.rap_write_reg(regs::AXSCtrl::def().with_write(true))
            .await?;

        let _ = with_timeout(Duration::from_millis(20), async {
            loop {
                let Ok(v) = self.rap_read_reg::<regs::AXSCtrl>().await else { continue; };
                if u8::from(v) == 0 {
                    break;
                }
            }
        })
        .await;

        self.clear_flags().await?;

        Ok(())
    }
}

/// rap reading
impl<SPI: SpiDevice, const DIAMETER: u32> Trackpad<SPI, DIAMETER> {
    async fn rap_read_reg<R: regs::Register<u8>>(&mut self) -> Result<R, SPI::Error> {
        let mut b: u8 = 0u8;
        self.rap_read(R::REG, core::slice::from_mut(&mut b)).await?;
        Ok(R::from_byte(b))
    }

    async fn rap_write_reg<R: regs::Register<u8>>(&mut self, value: R) -> Result<(), SPI::Error> {
        self.rap_write(R::REG, &[value.to_byte()]).await
    }

    // async fn rap_read_byte(&mut self, address: u8) -> Result<u8, SPI::Error> {
    //     let mut b: u8 = 0u8;
    //     self.rap_read(address, core::slice::from_mut(&mut b))
    //         .await?;
    //     Ok(b)
    // }

    // async fn rap_write_byte(&mut self, address: u8, value: u8) -> Result<(), SPI::Error> {
    //     self.rap_write(address, &[value]).await
    // }

    async fn rap_read(&mut self, address: u8, buf: &mut [u8]) -> Result<(), SPI::Error> {
        let cmd = address | READ_MASK;
        let mut bin = [0u8; 3];
        self.spi
            .transfer(&mut bin, &[cmd, FILLER_BYTE, FILLER_BYTE])
            .await?;
        for dst in buf {
            self.spi
                .transfer(core::slice::from_mut(dst), &[FILLER_BYTE])
                .await?;
        }
        Ok(())
    }

    async fn rap_write(&mut self, address: u8, buf: &[u8]) -> Result<(), SPI::Error> {
        let cmd = address | WRITE_MASK;
        self.spi.write_transaction(&[&[cmd], buf]).await
    }
}
