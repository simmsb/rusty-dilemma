#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, impl_trait_in_assoc_type)]

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rusty_dilemma::main(spawner).await;
}
