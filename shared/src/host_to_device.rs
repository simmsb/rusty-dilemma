use core::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::fw::FWCmd;
use crate::side::KeyboardSide;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HostToDevice {
    /// Side the message should end up on, if None then both
    pub target_side: Option<KeyboardSide>,
    pub msg: HostToDeviceMsg,
}

impl HostToDevice {
    pub fn targets_side(&self, side: KeyboardSide) -> bool {
        self.target_side.map_or(true, |s| side == s)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HostToDeviceMsg {
    FWCmd(FWCmd),
}
