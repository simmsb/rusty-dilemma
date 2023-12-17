use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_sync::channel::TrySendError;

use shared::device_to_host::DeviceToHost;

pub use channel::COMMANDS_FROM_HOST;
pub use device::MAX_PACKET_SIZE;
pub use hid::publish_mouse_report;

use crate::messages::TransmittedMessage;
use crate::utils::log;

pub mod channel;
pub mod device;
pub mod hid;
pub mod picotool;

pub type USBDriver = impl embassy_usb::driver::Driver<'static>;

static GUESSED_OS: once_cell::sync::OnceCell<embassy_os_guess::OS> =
    once_cell::sync::OnceCell::new();

pub fn guessed_host_os() -> Option<embassy_os_guess::OS> {
    GUESSED_OS.get().copied()
}

fn set_guesser(driver: Driver<'static, USB>) -> USBDriver {
    let guesser = embassy_os_guess::OSGuesser::new(|guess| {
        let _ = GUESSED_OS.set(guess);
    });
    guesser.wrap_driver(driver)
}

pub fn init(spawner: &Spawner, driver: Driver<'static, USB>) {
    log::info!("Initializing usb");

    let driver = set_guesser(driver);
    let mut builder = device::init_usb(driver);

    channel::init(spawner, &mut builder);
    picotool::init(&mut builder);
    hid::init(spawner, &mut builder);

    spawner.must_spawn(device::run_usb(builder));
}

pub async fn send_msg(msg: TransmittedMessage<DeviceToHost>) {
    if let Err(TrySendError::Full(msg)) = try_send_msg(msg) {
        // this is a bit of a hack, for messages destined over the usb serial we
        // allow non-reliable messages to be dropped here if the queue is full,
        // as it likely means that the serial channel is not open
        //
        // we shouldn't be sending anything other than unreliable messages over
        // here anyway
        if msg.timeout.is_none() {
            return;
        }

        channel::COMMANDS_TO_HOST.send(msg).await;
    }
}

pub fn try_send_msg(
    msg: TransmittedMessage<DeviceToHost>,
) -> Result<(), TrySendError<TransmittedMessage<DeviceToHost>>> {
    channel::COMMANDS_TO_HOST.try_send(msg)
}
