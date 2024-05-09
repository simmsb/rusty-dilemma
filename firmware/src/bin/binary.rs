#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, impl_trait_in_assoc_type)]

use embassy_executor::Spawner;

// #[embassy_executor::main]
// async fn main(spawner: Spawner) {
//     rusty_dilemma::main(spawner).await;
// }

use rusty_dilemma::utils::MeasuringExecutor;

#[embassy_executor::task]
async fn asyncmain(spawner: Spawner) {
    rusty_dilemma::main(spawner).await;
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let executor = static_cell::make_static!(MeasuringExecutor::new());
    // let executor = static_cell::make_static!(embassy_executor::Executor::new());

    executor.run(|spawner| {
        spawner.must_spawn(asyncmain(spawner.clone()));
    });
}
