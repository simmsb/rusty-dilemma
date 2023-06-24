use core::hash::Hash;
use serde::{Deserialize, Serialize};
use shared::{device_to_host::DeviceToHost, hid::MouseReport, host_to_device::HostToDeviceMsg};

use crate::rgb::animations::AnimationSync;


#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[cfg_attr(feature = "probe", derive(defmt::Format))]
pub enum MouseButton {
    LeftClick,
    RightClick,
}

impl MouseButton {
    pub fn bit(&self) -> u8 {
        match self {
            MouseButton::LeftClick =>  1 << 0,
            MouseButton::RightClick => 1 << 2,
        }
    }
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
    MousebuttonPress(MouseButton),
    MousebuttonRelease(MouseButton),
}
