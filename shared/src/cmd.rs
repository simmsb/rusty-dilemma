use core::hash::{Hash, Hasher};

use atomic_polyfill::AtomicU8;
use defmt::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, defmt::Format, Debug)]
pub struct Command<T> {
    pub uuid: u8,
    pub csum: u8,
    pub cmd: T,
}

pub fn csum<T: Hash>(v: T) -> u8 {
    let mut hasher = StableHasher::new(fnv::FnvHasher::default());
    v.hash(&mut hasher);
    let checksum = hasher.finish();

    let bytes = checksum.to_le_bytes();

    bytes.iter().fold(0, core::ops::BitXor::bitxor)
}

impl<T: Hash> Command<T> {
    pub fn new(cmd: T) -> Self {
        static UUID_GEN: AtomicU8 = AtomicU8::new(0);
        let uuid = UUID_GEN.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        let csum = csum((&cmd, uuid));
        Self { uuid, csum, cmd }
    }

    /// validate the data of the command
    /// though the data will probably fail to deserialize if it has been corrupted, this just makes sure
    pub fn validate(&self) -> bool {
        let csum = csum((&self.cmd, self.uuid));
        if csum == self.csum {
            true
        } else {
            debug!(
                "Invalid csum on {}, expected: {}, computed: {}",
                core::any::type_name::<Self>(),
                self.csum,
                csum
            );
            false
        }
    }

    pub fn ack(&self) -> Ack {
        let csum = csum(self.uuid);
        Ack {
            uuid: self.uuid,
            csum,
        }
    }
}

#[derive(Serialize, Deserialize, defmt::Format, Debug)]
pub struct Ack {
    pub uuid: u8,
    pub csum: u8,
}

#[derive(Serialize, Deserialize, defmt::Format, Debug)]
#[repr(u8)]
pub enum CmdOrAck<T> {
    Cmd(Command<T>),
    Ack(Ack),
}

impl Ack {
    pub fn validate(self) -> Option<Self> {
        let csum = csum(self.uuid);
        if csum == self.csum {
            Some(self)
        } else {
            None
        }
    }
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
