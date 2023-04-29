use embassy_executor::Spawner;
use embassy_futures::{
    select::{self, select},
    yield_now,
};
use embassy_rp::{
    gpio::{AnyPin, Flex, Pin},
    pio::{Pio0, PioStateMachine, PioStateMachineInstance, Sm0, Sm1},
    pio_instr_util,
    relocate::RelocatedProgram,
    Peripheral,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, pipe::Pipe};
use embassy_time::{Duration, Timer};

#[allow(unused_imports)]
use crate::utils::log;

pub static OTHER_SIDE_TX: Pipe<ThreadModeRawMutex, 16> = Pipe::new();
pub static OTHER_SIDE_RX: Pipe<ThreadModeRawMutex, 16> = Pipe::new();

pub const USART_SPEED: u32 = 19200;

pub fn init(spawner: &Spawner, tx_sm: SM<Sm0>, rx_sm: SM<Sm1>, pin: AnyPin) {
    spawner.must_spawn(half_duplex_task(tx_sm, rx_sm, pin));
}
pub type SM<S> = PioStateMachineInstance<Pio0, S>;

pub async fn enter_rx(
    tx_sm: &mut SM<Sm0>,
    rx_sm: &mut SM<Sm1>,
    pin: &mut Flex<'static, AnyPin>,
    p: &AnyPin,
) {
    while !tx_sm.is_tx_empty() {
        yield_now().await;
    }

    Timer::after(Duration::from_micros(1000000 * 11 / USART_SPEED as u64)).await;

    tx_sm.set_enable(false);
    pin.set_drive_strength(embassy_rp::gpio::Drive::_2mA);
    tx_sm.set_pindirs([(p.pin(), false)]);
    tx_sm.set_pins([(p.pin(), true)]);
    rx_sm.set_enable(true);
}

pub fn enter_tx(
    tx_sm: &mut SM<Sm0>,
    rx_sm: &mut SM<Sm1>,
    pin: &mut Flex<'static, AnyPin>,
    p: &AnyPin,
) {
    // okay so the way this works is that the pio doesn't actually set the pin
    // high or low, instead we toggle the input/output status of the pin to
    // switch it from a pull-high to a pull-low
    rx_sm.set_enable(false);
    rx_sm.set_pins([(p.pin(), false)]);
    pin.set_drive_strength(embassy_rp::gpio::Drive::_12mA);
    tx_sm.restart();
    tx_sm.set_enable(true);
}

#[embassy_executor::task]
pub async fn half_duplex_task(tx_sm: SM<Sm0>, rx_sm: SM<Sm1>, pin: AnyPin) {
    let mut flex = Flex::new(unsafe { pin.clone_unchecked() });
    flex.set_as_output();
    flex.set_pull(embassy_rp::gpio::Pull::Up);

    let mut tx_sm = half_duplex_task_tx(tx_sm, unsafe { pin.clone_unchecked() });
    let mut rx_sm = half_duplex_task_rx(rx_sm, unsafe { pin.clone_unchecked() });

    enter_rx(&mut tx_sm, &mut rx_sm, &mut flex, &pin).await;

    let mut buf = [0u8; 16];
    let reader = OTHER_SIDE_TX.reader();

    loop {
        match select(reader.read(&mut buf), rx_sm.wait_pull()).await {
            select::Either::First(n) => {
                // let now = Instant::now();
                // log::info!("sending bytes: {:08b}", &buf[..n]);
                enter_tx(&mut tx_sm, &mut rx_sm, &mut flex, &pin);
                for b in &buf[..n] {
                    tx_sm.wait_push(*b as u32).await;
                }
                enter_rx(&mut tx_sm, &mut rx_sm, &mut flex, &pin).await;
                // log::info!("sent bytes: {} in {}", &buf[..n], now.elapsed());
            }
            select::Either::Second(x) => {
                let x = x.to_be_bytes()[0];
                // log::info!("got byte: {:08b}: {}", 255 - x, 255 - x);
                OTHER_SIDE_RX.write(&[255 - x as u8]).await;
            }
        }
    }
}

const CLOCK_FREQ: u32 = 125_000_000;

const fn pio_divisor(freq: u32, div: u32) -> (u16, u8) {
    let int = freq / div;
    let rem = freq - (int * div);
    let frac = (rem * 256) / div;
    let int = if int == 65536 { 0 } else { int };
    (int as u16, frac as u8)
}

const PIO_FREQ: (u16, u8) = pio_divisor(CLOCK_FREQ, 8 * USART_SPEED);

pub fn half_duplex_task_tx(mut sm: SM<Sm0>, pin: AnyPin) -> SM<Sm0> {
    let tx_prog = pio_proc::pio_asm!(
        ".origin 0",
        ".wrap_target",
        "set   pindirs 0",
        "pull  block [6]",
        "set   pindirs 1",
        "set   x, 7  [6]",
        "bitloop:"
        "out   pindirs, 1",
        "jmp   x--, bitloop [6]",
        "set   pindirs, 0",
        ".wrap"
    );

    let relocated = RelocatedProgram::new(&tx_prog.program);

    let mut out_pin = sm.make_pio_pin(pin);
    out_pin.set_slew_rate(embassy_rp::gpio::SlewRate::Fast);
    out_pin.set_schmitt(true);

    let pio_pins = &[&out_pin];
    sm.set_set_pins(pio_pins);
    sm.set_out_pins(pio_pins);

    sm.set_pull_threshold(32);
    sm.set_out_shift_dir(embassy_rp::pio::ShiftDirection::Right);
    sm.set_autopull(false);

    sm.set_fifo_join(embassy_rp::pio::FifoJoin::TxOnly);

    sm.write_instr(relocated.origin() as usize, relocated.code());
    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    pio_instr_util::exec_jmp(&mut sm, relocated.origin());
    sm.set_clkdiv(PIO_FREQ.0, PIO_FREQ.1);
    sm.set_enable(true);

    sm
}

pub fn half_duplex_task_rx(mut sm: SM<Sm1>, pin: AnyPin) -> SM<Sm1> {
    let rx_prog = pio_proc::pio_asm!(
        ".origin 12",
        ".wrap_target",
        "start:",
        "wait  0 pin, 0",
        "set   x, 7         [10]",
        "bitloop:",
        "in    pins, 1",
        "jmp   x--, bitloop [6]",
        // "jmp   pin, stop",
        // "wait  1 pin, 0",
        // "jmp   start",
        // "stop:",
        "push block",
        ".wrap"
    );

    let relocated = RelocatedProgram::new(&rx_prog.program);

    let in_pin = sm.make_pio_pin(pin);

    sm.set_in_base_pin(&in_pin);
    sm.set_jmp_pin(in_pin.pin());

    sm.set_push_threshold(32);
    sm.set_in_shift_dir(embassy_rp::pio::ShiftDirection::Right);
    sm.set_autopush(false);

    sm.set_fifo_join(embassy_rp::pio::FifoJoin::RxOnly);

    sm.write_instr(relocated.origin() as usize, relocated.code());
    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    pio_instr_util::exec_jmp(&mut sm, relocated.origin());
    sm.set_clkdiv(PIO_FREQ.0, PIO_FREQ.1);
    sm.set_enable(true);

    sm
}
