use core::mem::MaybeUninit;

use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_usb::{
    control::{Recipient, RequestType},
    driver::Driver,
    types::{InterfaceNumber, StringIndex},
    Builder, Handler,
};

use crate::utils;

struct PicoToolClass {}

struct Control {
    comm_if: InterfaceNumber,
    str_idx: StringIndex,
}

struct State {
    control: MaybeUninit<Control>,
}

const CLASS_VENDOR_SPECIFIC: u8 = 0xFF;
const RESET_INTERFACE_SUBCLASS: u8 = 0x00;
const RESET_INTERFACE_PROTOCOL: u8 = 0x01;
const RESET_REQUEST_BOOTSEL: u8 = 0x01;

impl Handler for Control {
    fn get_string(
        &mut self,
        index: embassy_usb::types::StringIndex,
        _lang_id: u16,
    ) -> Option<&str> {
        (index == self.str_idx).then_some("Reset")
    }

    fn control_out(
        &mut self,
        req: embassy_usb::control::Request,
        _data: &[u8],
    ) -> Option<embassy_usb::control::OutResponse> {
        if !(req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index == u8::from(self.comm_if) as u16)
        {
            return None;
        }

        match req.request {
            RESET_REQUEST_BOOTSEL => {
                reset_to_usb_boot(1 << 17, 0);
                unreachable!();
            }
            _ => Some(embassy_usb::control::OutResponse::Rejected),
        }
    }

    fn control_in<'a>(
        &'a mut self,
        req: embassy_usb::control::Request,
        _buf: &'a mut [u8],
    ) -> Option<embassy_usb::control::InResponse<'a>> {
        if !(req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index == u8::from(self.comm_if) as u16)
        {
            return None;
        }

        Some(embassy_usb::control::InResponse::Rejected)
    }
}

impl PicoToolClass {
    fn new<'d, D: Driver<'d>>(builder: &mut Builder<'d, D>, state: &'d mut State) -> Self {
        let str_idx = builder.string();
        let mut func = builder.function(
            CLASS_VENDOR_SPECIFIC,
            RESET_INTERFACE_SUBCLASS,
            RESET_INTERFACE_PROTOCOL,
        );
        let mut iface = func.interface();
        let comm_if = iface.interface_number();
        let alt = iface.alt_setting(
            CLASS_VENDOR_SPECIFIC,
            RESET_INTERFACE_SUBCLASS,
            RESET_INTERFACE_PROTOCOL,
            Some(str_idx),
        );

        let handler = state.control.write(Control { comm_if, str_idx });

        drop(alt);
        drop(iface);
        drop(func);

        builder.handler(handler);

        Self {}
    }
}

pub fn init<'d, D: Driver<'d>>(builder: &mut Builder<'d, D>) {
    let state = utils::singleton!(State {
        control: MaybeUninit::uninit()
    });

    PicoToolClass::new(builder, state);
}
