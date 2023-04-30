use once_cell::sync::OnceCell;
use shared::side::KeyboardSide;

static SIDE: OnceCell<KeyboardSide> = OnceCell::new();
static HAS_USB: OnceCell<bool> = OnceCell::new();

pub fn init(side: KeyboardSide, has_usb: bool) {
    SIDE.set(side).unwrap();
    HAS_USB.set(has_usb).unwrap();
}

pub fn is_this_side(side: KeyboardSide) -> bool {
    get_side() == side
}

pub fn get_side() -> KeyboardSide {
    *SIDE.get().unwrap()
}

pub fn get_other_side() -> KeyboardSide {
    SIDE.get().unwrap().other()
}

pub fn this_side_has_usb() -> bool {
    *HAS_USB.get().unwrap()
}
