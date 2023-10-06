use core::{cell::RefCell, convert::Infallible, sync::atomic::AtomicBool};

use alloc::rc::Rc;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, RgbColor},
};
use embedded_hal_0_2::digital::v2::OutputPin;
use slint::platform::software_renderer as renderer;

use super::{draw_buffer::DrawBuffer, DISPLAY_SIZE};

pub struct PicoBackend<DrawBuffer> {
    off_flag: &'static AtomicBool,
    window: RefCell<Option<Rc<renderer::MinimalSoftwareWindow>>>,
    buffer_provider: RefCell<DrawBuffer>,
    start: embassy_time::Instant,
}

impl<DrawBuffer> PicoBackend<DrawBuffer> {
    pub fn new(off_flag: &'static AtomicBool, buffer_provider: DrawBuffer) -> Self {
        Self {
            off_flag,
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

        let mut is_off = false;

        loop {
            slint::platform::update_timers_and_animations();

            let should_be_off = self.off_flag.load(atomic_polyfill::Ordering::Relaxed);
            if should_be_off != is_off {
                is_off = should_be_off;

                if should_be_off {
                    let _ = self
                        .buffer_provider
                        .borrow_mut()
                        .display
                        .clear(Rgb565::BLACK);
                } else {
                    if let Some(window) = self.window.borrow().clone() {
                        window.request_redraw();
                        window.draw_if_needed(|renderer| {
                            let mut buffer_provider = self.buffer_provider.borrow_mut();
                            renderer
                                .set_repaint_buffer_type(renderer::RepaintBufferType::NewBuffer);
                            renderer
                                .set_repaint_buffer_type(renderer::RepaintBufferType::ReusedBuffer);
                            renderer.render_by_line(&mut *buffer_provider);
                        });
                    }
                }
            }

            if !is_off {
                if let Some(window) = self.window.borrow().clone() {
                    window.draw_if_needed(|renderer| {
                        let mut buffer_provider = self.buffer_provider.borrow_mut();
                        renderer.render_by_line(&mut *buffer_provider);
                    });

                    if window.has_active_animations() {
                        continue;
                    }
                }
            }
        }
    }
}
