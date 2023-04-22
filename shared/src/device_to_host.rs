use core::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::side::KeyboardSide;

pub const MAX_LOG_LEN: usize = 32;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub enum DeviceToHost {
    Log {
        from_side: KeyboardSide,
        msg: heapless::Vec<u8, MAX_LOG_LEN>,
    },
}
