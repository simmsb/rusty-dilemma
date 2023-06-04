use keyberon::action::{k, l, Action, HoldTapAction};
use keyberon::key_code::KeyCode;

use super::chord::Chorder;

pub const COLS_PER_SIDE: usize = 5;
pub const COLS: usize = COLS_PER_SIDE * 2;
pub const ROWS: usize = 5;
pub const N_LAYERS: usize = 3;

pub type CustomEvent = core::convert::Infallible;
pub type Layers = keyberon::layout::Layers<COLS, { ROWS + 1 }, N_LAYERS, CustomEvent>;
pub type Layout = keyberon::layout::Layout<COLS, { ROWS + 1 }, N_LAYERS, CustomEvent>;

const HOLD_TIMEOUT: u16 = 400;

const WIN_TAB: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: HOLD_TIMEOUT,
    hold: k(KeyCode::LGui),
    tap: k(KeyCode::Tab),
    config: keyberon::action::HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
});

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

const CTRL_Z: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: HOLD_TIMEOUT,
    hold: k(KeyCode::LCtrl),
    tap: k(KeyCode::Z),
    config: keyberon::action::HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
});

const SHIFT_A: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: HOLD_TIMEOUT,
    hold: k(KeyCode::LShift),
    tap: k(KeyCode::A),
    config: keyberon::action::HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
});

const CTRL_SLASH: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: HOLD_TIMEOUT,
    hold: k(KeyCode::RCtrl),
    tap: k(KeyCode::Slash),
    config: keyberon::action::HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
});

const SHIFT_SCOL: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: HOLD_TIMEOUT,
    hold: k(KeyCode::RShift),
    tap: k(KeyCode::SColon),
    config: keyberon::action::HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
});

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
        [(2, 6), (2, 7)] => [(5, 1)], // m + , = '
        [(2, 7), (2, 8)] => [(5, 2)]  // , + . = _
    )
}

macro_rules! m {
    ($($keys:expr),*) => {
        ::keyberon::action::m(&[$($keys),*].as_slice())
    };
}

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
        [{WIN_TAB} {L1_SP} LAlt n n n n BSpace {L2_SP} Enter],
        [Escape {m!(KeyCode::LAlt, KeyCode::X)} {m!(KeyCode::Space, KeyCode::Grave)} Delete < {m!(KeyCode::LShift, KeyCode::SColon)} > / '\\' '"'],
        [BSpace '\'' '_' n    n  n n   n      n n],
    }
    {
        [! @ '{' '}' | '`' ~ '\\' n '"' ],
        [# $ '(' ')' n  +  -  /   * '\''],
        [% ^ '[' ']' n  &  =  ,   . '_' ],
        [LAlt Space n n  n  n n n = n],
        [n n n n n n n  n  n n],
        [n n n n n n n  n  n n],
    }
    {
        [Kb1 Kb2 Kb3 Kb4 Kb5 Kb6 Kb7 Kb8 Kb9 Kb0],
        [F1  F2  F3  F4  F5  Left Down Up Right VolUp],
        [F6  F7  F8  F9  F10 PgDown {m!(KeyCode::LCtrl, KeyCode::Down)} {m!(KeyCode::LCtrl, KeyCode::Up)} PgUp VolDown],
        [F12 n F11   n t t n End Space n],
        [n = n   n   n n n    n   n n],
        [n n n   n   n n n    n   n n],
    }
};
