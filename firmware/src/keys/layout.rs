use keyberon::action::{k, l, Action, HoldTapAction};
use keyberon::key_code::KeyCode;

use crate::messages::device_to_device::MouseButton;

use super::chord::Chorder;

pub const COLS_PER_SIDE: usize = 5;
pub const COLS: usize = COLS_PER_SIDE * 2;
pub const ROWS: usize = 5;
pub const N_LAYERS: usize = 3;

pub type CustomEvent = MouseButton;
pub type Layers = keyberon::layout::Layers<COLS, { ROWS + 1 }, N_LAYERS, CustomEvent>;
pub type Layout = keyberon::layout::Layout<COLS, { ROWS + 1 }, N_LAYERS, CustomEvent>;

const HOLD_TIMEOUT: u16 = 400;

macro_rules! hold_tap {
    ($hold:expr, $tap:expr) => {
        Action::HoldTap(&HoldTapAction {
            timeout: HOLD_TIMEOUT,
            hold: k($hold),
            tap: $tap,
            config: keyberon::action::HoldTapConfig::PermissiveHold,
            tap_hold_interval: 0,
        })
    };
}

macro_rules! h_win {
    ($tap:expr) => {
        hold_tap!(KeyCode::LGui, $tap)
    };
}

macro_rules! h_lctrl {
    ($tap:expr) => {
        hold_tap!(KeyCode::LCtrl, $tap)
    };
}

macro_rules! h_rctrl {
    ($tap:expr) => {
        hold_tap!(KeyCode::RCtrl, $tap)
    };
}

macro_rules! h_lshift {
    ($tap:expr) => {
        hold_tap!(KeyCode::LShift, $tap)
    };
}

macro_rules! h_rshift {
    ($tap:expr) => {
        hold_tap!(KeyCode::RShift, $tap)
    };
}

const L1_SP: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: HOLD_TIMEOUT,
    hold: l(1),
    tap: k(KeyCode::Space),
    config: keyberon::action::HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
});

const L2_SP: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: HOLD_TIMEOUT,
    hold: l(2),
    tap: k(KeyCode::Space),
    config: keyberon::action::HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
});

type A = Action<CustomEvent>;

pub fn chorder() -> Chorder {
    dilemma_macros::chords!(
        [(0, 5), (0, 6)] => [(5, 0)],  // y + u = bspc
        [(0, 6), (0, 7)] => [(4, 3)],  // u + i = del
        [(0, 0), (0, 1)] => [(4, 0)],  // q + w = esc
        [(2, 1), (2, 2)] => [(4, 1)],  // x + c = M-x
        [(2, 2), (2, 3)] => [(4, 2)],  // c + v = spc, grave

        [(1, 5), (1, 6)] => [(4, 4)],  // h + j = <
        [(1, 6), (1, 7)] => [(4, 5)],  // j + k = :
        [(1, 7), (1, 8)] => [(4, 6)],  // k + l = >

        [(0, 7), (0, 8)] => [(4, 7)],  // i + o = \
        [(0, 8), (0, 9)] => [(4, 8)],  // o + p = /

        [(2, 5), (2, 6)] => [(4, 9)],  // n + m = "
        [(2, 6), (2, 7)] => [(5, 1)],  // m + , = '
        [(2, 7), (2, 8)] => [(5, 2)],  // , + . = _

        [(2, 0), (2, 2)] => [(5, 3)],  // ctrl + c = ctrl + c (mod tap thingy)
        [(2, 0), (2, 3)] => [(5, 4)],  // ctrl + v = ctrl + v (mod tap thingy)
    )
}

macro_rules! m {
    ($($keys:expr),*) => {
        ::keyberon::action::m(&[$($keys),*].as_slice())
    };
}

const WIN_TAB: A = h_win!(k(KeyCode::Tab));

const SHIFT_A: A = h_lshift!(k(KeyCode::A));
const CTRL_Z: A = h_lctrl!(k(KeyCode::Z));
const SHIFT_SCOL: A = h_rshift!(k(KeyCode::SColon));
const CTRL_SLASH: A = h_rctrl!(k(KeyCode::Slash));

const SHIFT_HASH: A = h_lshift!(keyberon::action::Action::MultipleKeyCodes(
    &[
        keyberon::key_code::KeyCode::LShift,
        keyberon::key_code::KeyCode::Kb3
    ]
    .as_slice()
));
const CTRL_PCT: A = h_lctrl!(keyberon::action::Action::MultipleKeyCodes(
    &[
        keyberon::key_code::KeyCode::LShift,
        keyberon::key_code::KeyCode::Kb5
    ]
    .as_slice()
));
const SHIFT_QUOT: A = h_rshift!(k(KeyCode::Quote));
const CTRL_UNDER: A = h_rctrl!(keyberon::action::Action::MultipleKeyCodes(
    &[
        keyberon::key_code::KeyCode::LShift,
        keyberon::key_code::KeyCode::Minus
    ]
    .as_slice()
));

const SHIFT_F1: A = h_lshift!(k(KeyCode::F1));
const CTRL_F6: A = h_lctrl!(k(KeyCode::F6));
const SHIFT_VOLUP: A = h_rshift!(k(KeyCode::VolUp));
const CTRL_VOLDOWN: A = h_rctrl!(k(KeyCode::VolDown));

// row 4 is weird
//
// x x 2 0 1  -  8 9 7 x x
// 3 4 2 0 0  -  0 0 7 5 6
#[rustfmt::skip]
pub static LAYERS: Layers  = keyberon::layout::layout! {
    {
        [Q W E R T Y U I O P],
        [{SHIFT_A} S D F G H J K L {SHIFT_SCOL}],
        [{CTRL_Z} X C V B N M , . {CTRL_SLASH}],
        [{WIN_TAB} {L1_SP} LAlt n n n n RAlt {L2_SP} Enter],
        [Escape {m!(KeyCode::LAlt, KeyCode::X)} {m!(KeyCode::Space, KeyCode::Grave)} Delete < {m!(KeyCode::LShift, KeyCode::SColon)} > / '\\' '"'],
        [BSpace '\'' '_' {m!(KeyCode::LCtrl, KeyCode::C)} {m!(KeyCode::LCtrl, KeyCode::V)}  n n   n      n n],
    }
    {
        [! @ '{' '}' | '`' ~ '\\' n '"' ],
        [{SHIFT_HASH} $ '(' ')' n  +  -  /   * {SHIFT_QUOT}],
        [{CTRL_PCT} ^ '[' ']' n  &  =  ,   . {CTRL_UNDER}],
        [LAlt Space n n  n  n n n = n],
        [n n n {Action::Custom(MouseButton::RightClick)} n n n  n  n n],
        [{Action::Custom(MouseButton::LeftClick)} n n n n n n  n  n n],
    }
    {
        [Kb1 Kb2 Kb3 Kb4 Kb5 Kb6 Kb7 Kb8 Kb9 Kb0],
        [{SHIFT_F1}  F2  F3  F4  F5  Left Down Up Right {SHIFT_VOLUP}],
        [{CTRL_F6}  F7  F8  F9  F10 PgDown {m!(KeyCode::LCtrl, KeyCode::Down)} {m!(KeyCode::LCtrl, KeyCode::Up)} PgUp {CTRL_VOLDOWN}],
        [F12 n F11   n t t n End Space n],
        [n = n   n   n n n    n   n n],
        [n n n   n   n n n    n   n n],
    }
};
