use portable_atomic::AtomicBool;
use shared::side::KeyboardSide;

static SIDE_IS_LEFT: AtomicBool = AtomicBool::new(false);
static HAS_USB: AtomicBool = AtomicBool::new(false);

pub fn init(side: KeyboardSide, has_usb: bool) {
    SIDE_IS_LEFT.store(side.is_left(), portable_atomic::Ordering::Relaxed);
    HAS_USB.store(has_usb, portable_atomic::Ordering::Relaxed);
}

pub fn is_this_side(side: KeyboardSide) -> bool {
    get_side() == side
}

pub fn get_side() -> KeyboardSide {
    if SIDE_IS_LEFT.load(portable_atomic::Ordering::Relaxed) {
        KeyboardSide::Left
    } else {
        KeyboardSide::Right
    }
}

pub fn get_other_side() -> KeyboardSide {
    get_side().other()
}

pub fn this_side_has_usb() -> bool {
    HAS_USB.load(portable_atomic::Ordering::Relaxed)
}
