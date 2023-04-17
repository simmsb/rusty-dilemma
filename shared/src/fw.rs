use core::hash::Hash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, defmt::Format, Hash, Clone, Debug)]
pub enum FWCmd {
    Prepare,
    Commit,
    WriteChunk {
        offset: u32,
        buf: heapless::Vec<u8, 64>,
    },
}
