use core::fmt::Write as _;

use embassy_time::Duration;
use log::{Metadata, Record};
use shared::device_to_host::{DeviceToHost, MAX_LOG_LEN};

use crate::usb;
use crate::utils::singleton;

pub fn setup_logger() {
    let logger: &mut Logger = singleton!(Logger);
    let logger = logger as &dyn log::Log;
    unsafe {
        let _ = log::set_logger_racy(logger).map(|()| log::set_max_level(log::LevelFilter::Info));
    }
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let _ = write!(Writer, "{}\r\n", record.args());
        }
    }

    fn flush(&self) {}
}

struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        let vec = heapless::Vec::from_slice(&s.as_bytes()[..MAX_LOG_LEN])
            .ok()
            .expect("Log slice was too big for vec");
        let cmd = DeviceToHost::Log {
            from_side: shared::side::KeyboardSide::Left,
            msg: vec,
        };
        let _ = usb::COMMANDS_TO_HOST.try_send((cmd, Duration::from_millis(100)));
        Ok(())
    }
}
