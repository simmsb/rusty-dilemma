use once_cell::sync::OnceCell;
use shared::side::KeyboardSide;

pub static SIDE: OnceCell<KeyboardSide> = OnceCell::new();
pub static HAS_USB: OnceCell<bool> = OnceCell::new();

pub fn init(side: KeyboardSide, has_usb: bool) {
    SIDE.set(side).unwrap();
    HAS_USB.set(has_usb).unwrap();
}

pub fn is_this_side(side: KeyboardSide) -> bool {
    *SIDE.get().unwrap() == side
}

pub fn this_side_has_usb() -> bool {
    *HAS_USB.get().unwrap()
}
