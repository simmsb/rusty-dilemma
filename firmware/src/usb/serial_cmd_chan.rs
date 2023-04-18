use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_rp::peripherals::USB;
use embassy_sync::channel::Channel;
use embassy_sync::pipe::{Pipe, Reader, Writer};
use embassy_sync::pubsub::PubSubChannel;
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::driver::Driver;
use embassy_usb::Builder;
use shared::device_to_host::DeviceToHost;
use shared::host_to_device::HostToDevice;

use crate::messages::TransmittedMessage;
use crate::messages::transmissions::Eventer;
use crate::utils;

use super::MAX_PACKET_SIZE;

pub static COMMANDS_FROM_HOST: PubSubChannel<CS, HostToDevice, 4, 4, 1> = PubSubChannel::new();
pub static COMMANDS_TO_HOST: Channel<CS, TransmittedMessage<DeviceToHost>, 8> = Channel::new();

const BUF_SIZE: usize = 128;

#[embassy_executor::task]
async fn serial_in_task(
    out_pipe: Writer<'static, CS, BUF_SIZE>,
    mut serial_rx: Receiver<'static, embassy_rp::usb::Driver<'static, USB>>,
) {
    loop {
        let mut rx: [u8; MAX_PACKET_SIZE as usize] = [0; MAX_PACKET_SIZE as usize];
        serial_rx.wait_connection().await;
        loop {
            if let Ok(len) = serial_rx.read_packet(&mut rx[..]).await {
                let _ = out_pipe.write(&rx[..len]).await;
            } else {
                break;
            }
        }
    }
}

#[embassy_executor::task]
async fn serial_out_task(
    in_pipe: Reader<'static, CS, BUF_SIZE>,
    mut serial_tx: Sender<'static, embassy_rp::usb::Driver<'static, USB>>,
) {
    loop {
        let mut rx: [u8; MAX_PACKET_SIZE as usize] = [0; MAX_PACKET_SIZE as usize];
        serial_tx.wait_connection().await;
        loop {
            let len = in_pipe.read(&mut rx[..]).await;
            if let Err(_) = serial_tx.write_packet(&rx[..len]).await {
                break;
            }
        }
    }
}

#[embassy_executor::task]
async fn eventer_task(tx: Writer<'static, CS, BUF_SIZE>, rx: Reader<'static, CS, BUF_SIZE>) {
    let mut eventer = Eventer::new(tx, rx);
    let (a, b, c) = eventer.split_tasks(&COMMANDS_TO_HOST, COMMANDS_FROM_HOST.publisher().unwrap());
    join3(a, b, c).await;
}

pub fn start_static_serial(
    spawner: &Spawner,
    builder: &mut Builder<'static, embassy_rp::usb::Driver<'static, USB>>,
) {
    let cdc_state = utils::singleton!(State::new());
    static FROM_USB_PIPE: Pipe<CS, BUF_SIZE> = Pipe::new();
    static TO_USB_PIPE: Pipe<CS, BUF_SIZE> = Pipe::new();

    let state = make_state(cdc_state, builder);
    let (serial_tx, serial_rx) = state.class.split();

    spawner.must_spawn(serial_in_task(FROM_USB_PIPE.writer(), serial_rx));
    spawner.must_spawn(serial_out_task(TO_USB_PIPE.reader(), serial_tx));
    spawner.must_spawn(eventer_task(TO_USB_PIPE.writer(), FROM_USB_PIPE.reader()));
}

pub fn make_state<'d, D>(
    cdc_state: &'d mut State<'d>,
    builder: &mut Builder<'d, D>,
) -> SerialState<'d, D>
where
    D: Driver<'d>,
{
    // Create classes on the builder.
    let class = CdcAcmClass::new(builder, cdc_state, MAX_PACKET_SIZE as u16);

    SerialState { class }
}

type CS = embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

pub struct SerialState<'d, D: Driver<'d>> {
    class: CdcAcmClass<'d, D>,
}
