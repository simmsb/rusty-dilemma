use embassy_rp::peripherals::USB;
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, Config};

pub const MAX_PACKET_SIZE: u16 = 64;

pub struct State {
    device_descriptor: [u8; 32],
    config_descriptor: [u8; 128],
    bos_descriptor: [u8; 16],
    control_buf: [u8; 64],
}

impl State {
    pub const fn new() -> State {
        State {
            device_descriptor: [0u8; 32],
            config_descriptor: [0u8; 128],
            bos_descriptor: [0u8; 16],
            control_buf: [0u8; 64],
        }
    }
}

pub fn init_usb<'d, D: Driver<'d>>(driver: D, state: &'d mut State) -> Builder<'d, D> {
    let mut config = Config::new(0x2e8a, 0x000a);
    config.manufacturer = Some("Ben Simms");
    config.product = Some("Dilemma");
    config.serial_number = None;
    config.max_power = 500;
    config.max_packet_size_0 = MAX_PACKET_SIZE as u8;

    // Required for windows compatiblity.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    Builder::new(
        driver,
        config,
        &mut state.device_descriptor,
        &mut state.config_descriptor,
        &mut state.bos_descriptor,
        &mut state.control_buf,
    )
}

#[embassy_executor::task]
pub async fn run_usb(builder: Builder<'static, embassy_rp::usb::Driver<'static, USB>>) {
    let mut device = builder.build();
    device.run().await;
}
