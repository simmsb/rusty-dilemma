use core::fmt::Write as _;

use log::{Metadata, Record};
use shared::device_to_host::{DeviceToHost, MAX_LOG_LEN};

use crate::messages::unreliable_msg;
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
        for chunk in s.as_bytes().chunks(MAX_LOG_LEN) {
            let vec = heapless::Vec::from_slice(chunk)
                .ok()
                .expect("Log slice was too big for vec");

            let cmd = DeviceToHost::Log {
                from_side: shared::side::KeyboardSide::Left,
                msg: vec,
            };

            let _ = usb::COMMANDS_TO_HOST.try_send(unreliable_msg(cmd));
        }
        Ok(())
    }
}
