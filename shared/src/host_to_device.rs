use core::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::fw::FWCmd;
use crate::side::KeyboardSide;

#[derive(Serialize, Deserialize, Eq, PartialEq, defmt::Format, Hash, Clone, Debug)]
pub struct HostToDevice {
    pub target_side: KeyboardSide,
    pub msg: HostToDeviceMsg,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, defmt::Format, Hash, Clone, Debug)]
pub enum HostToDeviceMsg {
    FWCmd(FWCmd),
}
