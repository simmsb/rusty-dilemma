use once_cell::sync::OnceCell;
use shared::side::KeyboardSide;

pub static SIDE: OnceCell<KeyboardSide> = OnceCell::new();
pub static HAS_USB: OnceCell<bool> = OnceCell::new();

pub fn init_left(has_usb: bool) {
    SIDE.set(KeyboardSide::Left).unwrap();
    HAS_USB.set(has_usb).unwrap();
}

pub fn init_right(has_usb: bool) {
    SIDE.set(KeyboardSide::Right).unwrap();
    HAS_USB.set(has_usb).unwrap();
}

pub fn is_this_side(side: KeyboardSide) -> bool {
    return *SIDE.get().unwrap() == side;
}
