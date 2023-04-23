use core::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::side::KeyboardSide;

pub const MAX_LOG_LEN: usize = 32;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceToHost {
    pub from_side: KeyboardSide,
    pub msg: DeviceToHostMsg,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceToHostMsg {
    Log { msg: heapless::Vec<u8, MAX_LOG_LEN> },
}
