use embassy_boot_rp::{AlignedBuffer, FirmwareUpdater, Partition};
use embassy_futures::select::{self, Either};
use embassy_rp::flash::Flash;
use embassy_rp::peripherals::{FLASH, WATCHDOG};
use embassy_rp::watchdog::Watchdog;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use shared::fw::FWCmd;

const FLASH_SIZE: usize = 2 * 1024 * 1024;
pub static FW_CMD_CHANNEL: Channel<ThreadModeRawMutex, FWCmd, 1> = Channel::new();

#[embassy_executor::task]
pub async fn updater_task(watchdog: WATCHDOG, flash: FLASH) {
    let mut watchdog = Watchdog::new(watchdog);

    watchdog.start(Duration::from_secs(8));

    let mut flash: Flash<_, FLASH_SIZE> = Flash::new(flash);
    let mut updater = FirmwareUpdater::default();

    let mut aligned_buf = AlignedBuffer([0; 32]);
    let mut writer: Option<Partition> = None;

    if updater
        .get_state_blocking(&mut flash, &mut aligned_buf.0[..1])
        .unwrap()
        == embassy_boot_rp::State::Swap
    {
        updater
            .mark_booted_blocking(&mut flash, &mut aligned_buf.0[..1])
            .unwrap();
    }

    loop {
        let t = Timer::after(Duration::from_secs(4));
        let r = FW_CMD_CHANNEL.recv();

        if let Either::First(msg) = select::select(r, t).await {
            match msg {
                FWCmd::Commit => {
                    updater
                        .mark_updated_blocking(&mut flash, &mut aligned_buf.0[..1])
                        .unwrap();
                    defmt::info!("Marked update, rebooting");
                    Timer::after(Duration::from_millis(500)).await;
                    cortex_m::peripheral::SCB::sys_reset();
                }
                FWCmd::Prepare => {
                    writer = Some(updater.prepare_update_blocking(&mut flash).unwrap());
                    defmt::info!("Prepping for update");
                }
                FWCmd::WriteChunk { offset, buf } => {
                    aligned_buf.0[..buf.len()].copy_from_slice(buf.as_slice());
                    if let Some(w) = &writer {
                        w.write_blocking(&mut flash, offset, &aligned_buf.0)
                            .unwrap();
                    }
                }
            }
        }

        watchdog.feed();
    }
}
