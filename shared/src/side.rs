use core::hash::Hash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeyboardSide {
    Left,
    Right,
}

impl KeyboardSide {
    pub fn is_left(self) -> bool {
        self == Self::Left
    }
    pub fn is_right(self) -> bool {
        self == Self::Right
    }
    pub fn other(self) -> Self {
        match self {
            KeyboardSide::Left => KeyboardSide::Right,
            KeyboardSide::Right => KeyboardSide::Left,
        }
    }
}
