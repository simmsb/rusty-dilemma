use core::{
    hash::{Hash, Hasher},
    u8,
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "defmt")]
use defmt::debug;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Command<T> {
    pub reliable: bool,
    pub cmd: T,
    pub csum: u16,
}

pub fn calc_csum<T: Hash>(v: T) -> u16 {
    let mut hasher = StableHasher::new(crc32fast::Hasher::new());
    v.hash(&mut hasher);
    let checksum = hasher.finish();

    let [a, b, c, d, e, f, g, h] = checksum.to_le_bytes();
    let (a, b, c, d) = (
        u16::from_le_bytes([a, b]),
        u16::from_le_bytes([c, d]),
        u16::from_le_bytes([e, f]),
        u16::from_le_bytes([g, h]),
    );

    a ^ b ^ c ^ d
}

impl<T: Hash> Command<T> {
    pub fn new_reliable(cmd: T) -> Self {
        let csum = calc_csum(&cmd);
        Self {
            reliable: true,
            cmd,
            csum,
        }
    }

    pub fn new_unreliable(cmd: T) -> Self {
        let csum = calc_csum(&cmd);
        Self {
            reliable: false,
            cmd,
            csum,
        }
    }

    /// validate the data of the command
    /// though the data will probably fail to deserialize if it has been corrupted, this just makes sure
    pub fn validate(&self) -> bool {
        let expected_csum = calc_csum(&self.cmd);
        self.csum == expected_csum
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CmdOrAck<T> {
    Cmd(Command<T>),
    Ack,
}

#[derive(Debug, Default)]
struct StableHasher<T> {
    inner: T,
}

impl<T: Hasher> Hasher for StableHasher<T> {
    fn write_u8(&mut self, i: u8) {
        self.write(&[i])
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_le_bytes())
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_le_bytes())
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_le_bytes())
    }

    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_le_bytes())
    }

    fn write_usize(&mut self, i: usize) {
        let bytes = i.to_le_bytes().iter().fold(0, core::ops::BitXor::bitxor);
        self.write(&[bytes])
    }

    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8)
    }

    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16)
    }

    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32)
    }

    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64)
    }

    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128)
    }

    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize)
    }

    fn finish(&self) -> u64 {
        self.inner.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.inner.write(bytes);
    }
}

impl<T> StableHasher<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}
