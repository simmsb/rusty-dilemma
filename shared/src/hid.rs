use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HidReport {
    Mouse(MouseReport),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MouseReport {
    pub x: i8,
    pub y: i8,
    pub wheel: i8,
    pub pan: i8,
}

// TODO: kbreport
