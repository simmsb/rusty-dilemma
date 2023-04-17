use embassy_executor::Spawner;

pub mod distributors;
pub mod transmissions;

pub fn init(spawner: &Spawner) {
    transmissions::init_pool();

    spawner.must_spawn(distributors::from_usb_distributor());
}
