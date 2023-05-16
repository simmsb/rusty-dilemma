#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    rusty_dilemma::entry();
}
