// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

//! The `display-interface-spi` crate cannot be used because it doesn't "flush" the spi between
//! the write and the changes in the CS and DC pin. This results in artifacts being shown on the
//! screen
//!
//! Work-around the problem by using `transfer` instead of write.

use embedded_hal_1 as hal;
use hal::digital::OutputPin;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

fn send_u8<SPI: hal::spi::SpiDevice<u8>>(
    spi: &mut SPI,
    words: DataFormat<'_>,
) -> Result<(), DisplayError> {
    match words {
        DataFormat::U8Iter(iter) => {
            let mut buf = [0; 32];
            let mut i = 0;

            for v in iter.into_iter() {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(&mut buf)
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&mut buf[..i])
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        DataFormat::U16BEIter(iter) => {
            for mut v in iter.map(u16::to_be_bytes) {
                spi.write(&mut v).map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command as well as a chip-select pin
pub struct SPIInterfaceNoCS<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI, DC> SPIInterfaceNoCS<SPI, DC>
where
    SPI: hal::spi::SpiDevice<u8>,
    DC: OutputPin,
{
    /// Create new SPI interface for communication with a display driver
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

impl<SPI, DC> WriteOnlyDataCommand for SPIInterfaceNoCS<SPI, DC>
where
    SPI: hal::spi::SpiDevice<u8>,
    DC: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let err = send_u8(&mut self.spi, cmds);

        // ---
        // for _ in 0..70 {
        //     self.cs.set_low().map_err(|_| DisplayError::CSError)?;
        // }
        // ---

        err
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let err = send_u8(&mut self.spi, buf);

        err
    }
}
