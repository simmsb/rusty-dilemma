use core::{cell::RefCell, convert::Infallible};

use alloc::rc::Rc;
use embedded_hal_0_2::digital::v2::OutputPin;
use embedded_hal_1::delay::DelayUs;
use slint::platform::software_renderer as renderer;

use super::{draw_buffer::DrawBuffer, DISPLAY_SIZE};

pub struct PicoBackend<DrawBuffer> {
    window: RefCell<Option<Rc<renderer::MinimalSoftwareWindow>>>,
    buffer_provider: RefCell<DrawBuffer>,
    start: embassy_time::Instant,
}

impl<DrawBuffer> PicoBackend<DrawBuffer> {
    pub fn new(buffer_provider: DrawBuffer) -> Self {
        Self {
            window: Default::default(),
            buffer_provider: buffer_provider.into(),
            start: embassy_time::Instant::now(),
        }
    }
}

impl<DI: display_interface::WriteOnlyDataCommand, RST: OutputPin<Error = Infallible>>
    slint::platform::Platform
    for PicoBackend<DrawBuffer<mipidsi::Display<DI, mipidsi::models::ST7789, RST>>>
{
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        let window =
            renderer::MinimalSoftwareWindow::new(renderer::RepaintBufferType::ReusedBuffer);
        self.window.replace(Some(window.clone()));
        Ok(window)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        self.start.elapsed().into()
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        self.window
            .borrow()
            .as_ref()
            .unwrap()
            .set_size(DISPLAY_SIZE);

        loop {
            slint::platform::update_timers_and_animations();

            if let Some(window) = self.window.borrow().clone() {
                window.draw_if_needed(|renderer| {
                    let mut buffer_provider = self.buffer_provider.borrow_mut();
                    renderer.render_by_line(&mut *buffer_provider);
                });

                if window.has_active_animations() {
                    continue;
                }
            }

            if let Some(d) = slint::platform::duration_until_next_timer_update() {
                let micros = d.as_micros() as u32;
                if micros < 10 {
                    // Cannot wait for less than 10µs, or `schedule()` panics
                    continue;
                } else {
                    embassy_time::Delay.delay_us(micros);
                }
            }
        }
    }
}
