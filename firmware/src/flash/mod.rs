use core::any::TypeId;

use ekv::flash::{self, PageID};
use ekv::{config, Database};
use embassy_rp::flash::Flash;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};
use once_cell::sync::OnceCell;
use rand::Rng;

use crate::rng::MyRng;

static DB: OnceCell<
    Database<
        DbFlash<Flash<embassy_rp::peripherals::FLASH, embassy_rp::flash::Async, FLASH_SIZE>>,
        ThreadModeRawMutex,
    >,
> = OnceCell::new();

pub async fn init(flash: embassy_rp::peripherals::FLASH, dma: embassy_rp::dma::AnyChannel) {
    let flash = DbFlash {
        flash: Flash::new(flash, dma),
        start: unsafe { &__config_start as *const u32 as usize },
    };
    let mut cfg = ekv::Config::default();
    cfg.random_seed = MyRng.gen();
    let db = Database::new(flash, cfg);

    if db.mount().await.is_err() {
        if db.format().await.is_err() {
            return;
        }
    }

    DB.set(db).ok().unwrap();
}

async fn set<T: core::any::Any + serde::Serialize>(value: &T) -> Option<()> {
    let mut buf = [0u8; ekv::config::MAX_VALUE_SIZE];
    let buf = postcard::to_slice(value, &mut buf).ok()?;
    let mut tx = DB.get().unwrap().write_transaction().await;

    // convert the typeid of the key to a byte array
    let key = unsafe {
        core::mem::transmute::<_, [u8; core::mem::size_of::<TypeId>()]>(TypeId::of::<T>())
    };

    tx.write(&key, &buf).await.ok()?;
    tx.commit().await.ok()?;

    Some(())
}

async fn get<T: core::any::Any + serde::de::DeserializeOwned>() -> Option<T> {
    let mut buf = [0u8; ekv::config::MAX_VALUE_SIZE];

    let mut tx = DB.get().unwrap().read_transaction().await;

    // convert the typeid of the key to a byte array
    let key = unsafe {
        core::mem::transmute::<_, [u8; core::mem::size_of::<TypeId>()]>(TypeId::of::<T>())
    };

    let len = tx.read(&key, &mut buf).await.ok()?;

    postcard::from_bytes(&buf[..len]).ok()
}

#[cfg(feature = "m2")]
const FLASH_SIZE: usize = 256 * 1024;
#[cfg(not(feature = "m2"))]
const FLASH_SIZE: usize = 8192 * 1024;

extern "C" {
    // u32 as align is 4
    static __config_start: u32;
}

// Workaround for alignment requirements.
#[repr(C, align(4))]
struct AlignedBuf<const N: usize>([u8; N]);

struct DbFlash<T: NorFlash + ReadNorFlash> {
    start: usize,
    flash: T,
}

impl<T: NorFlash + ReadNorFlash> flash::Flash for DbFlash<T> {
    type Error = T::Error;

    fn page_count(&self) -> usize {
        config::MAX_PAGE_COUNT
    }

    async fn erase(&mut self, page_id: PageID) -> Result<(), <DbFlash<T> as flash::Flash>::Error> {
        self.flash
            .erase(
                (self.start + page_id.index() * config::PAGE_SIZE) as u32,
                (self.start + page_id.index() * config::PAGE_SIZE + config::PAGE_SIZE) as u32,
            )
            .await
    }

    async fn read(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &mut [u8],
    ) -> Result<(), <DbFlash<T> as flash::Flash>::Error> {
        let address = self.start + page_id.index() * config::PAGE_SIZE + offset;
        let mut buf = AlignedBuf([0; config::PAGE_SIZE]);
        self.flash
            .read(address as u32, &mut buf.0[..data.len()])
            .await?;
        data.copy_from_slice(&buf.0[..data.len()]);
        Ok(())
    }

    async fn write(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &[u8],
    ) -> Result<(), <DbFlash<T> as flash::Flash>::Error> {
        let address = self.start + page_id.index() * config::PAGE_SIZE + offset;
        let mut buf = AlignedBuf([0; config::PAGE_SIZE]);
        buf.0[..data.len()].copy_from_slice(data);
        self.flash.write(address as u32, &buf.0[..data.len()]).await
    }
}
