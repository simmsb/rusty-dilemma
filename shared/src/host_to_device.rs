use core::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::fw::FWCmd;
use crate::side::KeyboardSide;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HostToDevice {
    pub target_side: KeyboardSide,
    pub msg: HostToDeviceMsg,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HostToDeviceMsg {
    FWCmd(FWCmd),
}
