use core::hash::Hash;
use serde::{Deserialize, Serialize};

pub const FW_CHUNK_SIZE: usize = 32;

#[derive(Serialize, Deserialize, Eq, PartialEq, defmt::Format, Hash, Clone, Debug)]
pub enum FWCmd {
    Prepare,
    Commit,
    WriteChunk {
        offset: u32,
        buf: heapless::Vec<u8, FW_CHUNK_SIZE>,
    },
}
