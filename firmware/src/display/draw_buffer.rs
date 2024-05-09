use core::convert::Infallible;

use embedded_hal_1 as hal;
use hal::digital::OutputPin;
use slint::platform::software_renderer::LineBufferProvider;

use super::TargetPixel;

pub struct DrawBuffer<Display> {
    pub display: Display,
    pub buffer: &'static mut [TargetPixel],
}

impl<DI: display_interface::WriteOnlyDataCommand, RST: OutputPin<Error = Infallible>>
    LineBufferProvider for &mut DrawBuffer<mipidsi::Display<DI, mipidsi::models::ST7789, RST>>
{
    type TargetPixel = TargetPixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [TargetPixel]),
    ) {
        render_fn(&mut self.buffer[range.clone()]);

        let _ = self.display.set_pixels(
            range.start as u16,
            line as u16,
            range.end as u16,
            line as u16,
            self.buffer[range]
                .iter()
                .map(|x| unsafe { core::mem::transmute(*x) }),
        );
    }
}
