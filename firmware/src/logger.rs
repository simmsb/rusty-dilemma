use core::cell::RefCell;
use core::fmt::Write;

use bbqueue::{BBBuffer, Consumer, Producer};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use log_log::{Metadata, Record};
use shared::device_to_host::{DeviceToHostMsg, MAX_LOG_LEN};

use crate::messages::distributors::MessageProvenance;
use crate::messages::{self, unreliable_msg};
use crate::utils::singleton;

static BB: BBBuffer<256> = BBBuffer::new();

pub fn setup_logger() {
    let logger: &mut Logger = singleton!(Logger::init());
    let logger = logger as &dyn log_log::Log;
    unsafe {
        let _ = log_log::set_logger_racy(logger).map(|()| log_log::set_max_level(log_log::LevelFilter::Info));
    }
}

struct LoggerInner {
    producer: Producer<'static, 256>,
    consumer: Consumer<'static, 256>,
}

struct Logger(Mutex<ThreadModeRawMutex, RefCell<LoggerInner>>);

impl Logger {
    fn init() -> Self {
        let (p, c) = BB.try_split().unwrap();

        let inner = LoggerInner {
            producer: p,
            consumer: c,
        };

        Self(Mutex::new(RefCell::new(inner)))
    }
}

impl log_log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut tmp = heapless::String::<128>::new();
            let _ = write!(&mut tmp, "{}\r\n", record.args());
            let src = tmp.as_bytes();
            self.0.lock(|i| {
                let mut i = i.borrow_mut();
                let Ok(mut grant) = i.producer.grant_max_remaining(src.len()) else { return };
                let buf = grant.buf();
                let write_len = src.len().min(buf.len());
                buf[..write_len].copy_from_slice(&src[..write_len]);
                grant.commit(write_len);

                // do this again with any remaining in case we were at the end of the circular buffer
                let src = &src[write_len..];

                let Ok(mut grant) = i.producer.grant_max_remaining(src.len()) else { return };
                let buf = grant.buf();
                let write_len = src.len().min(buf.len());
                buf[..write_len].copy_from_slice(&src[..write_len]);
                grant.commit(write_len);
            });
        }
        self.flush();
    }

    fn flush(&self) {
        self.0.lock(|i| {
            let mut i = i.borrow_mut();
            'outer: while let Ok(grant) = i.consumer.read() {
                let mut emitted = 0;
                for chunk in grant.buf().chunks(MAX_LOG_LEN) {
                    let vec = heapless::Vec::from_slice(chunk)
                        .ok()
                        .expect("Log slice was too big for vec");

                    let cmd = DeviceToHostMsg::Log { msg: vec };

                    if messages::try_send_to_host(unreliable_msg(cmd), MessageProvenance::Origin)
                        .is_some()
                    {
                        emitted += chunk.len();
                    } else {
                        grant.release(emitted);
                        break 'outer;
                    }
                }
                grant.release(emitted);
            }
        });
    }
}
