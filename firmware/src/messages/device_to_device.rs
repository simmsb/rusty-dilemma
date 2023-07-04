use core::hash::Hash;
use serde::{Deserialize, Serialize};
use shared::{device_to_host::DeviceToHost, hid::MouseReport, host_to_device::HostToDeviceMsg};

use crate::rgb::animations::AnimationSync;


#[cfg_attr(feature = "probe", derive(defmt::Format))]
#[bitfield_struct::bitfield(u8)]
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct MouseState {
    pub left: bool,
    pub right: bool,
    pub scrolling: bool,
    #[bits(5)]
    _padding: u8,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "probe", derive(defmt::Format))]
pub enum DeviceToDevice {
    Ping,
    Pong,
    ForwardedFromHost(HostToDeviceMsg),
    ForwardedToHost(DeviceToHost),
    ForwardedToHostMouse(MouseReport),
    KeyPress(u8, u8),
    KeyRelease(u8, u8),
    SetAnimation(AnimationSync),
    SyncAnimation(AnimationSync),
    SyncMouseState(MouseState),
}
