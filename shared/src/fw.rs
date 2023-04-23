use core::hash::Hash;
use serde::{Deserialize, Serialize};

pub const FW_CHUNK_SIZE: usize = 32;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FWCmd {
    Prepare,
    Commit,
    WriteChunk {
        offset: u32,
        buf: heapless::Vec<u8, FW_CHUNK_SIZE>,
    },
}
