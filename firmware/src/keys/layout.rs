use keyberon::action::{k, l, Action, HoldTapAction};
use keyberon::chording::ChordDef;
use keyberon::key_code::KeyCode;

pub const COLS_PER_SIDE: usize = 5;
pub const COLS: usize = COLS_PER_SIDE * 2;
pub const ROWS: usize = 5;
pub const N_LAYERS: usize = 3;

pub type CustomEvent = core::convert::Infallible;
pub type Layers = keyberon::layout::Layers<COLS, { ROWS + 1 }, N_LAYERS, CustomEvent>;
pub type Layout = keyberon::layout::Layout<COLS, { ROWS + 1 }, N_LAYERS, CustomEvent>;

const ALT_TAB: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: 200,
    hold: k(KeyCode::LAlt),
    tap: k(KeyCode::Tab),
    config: keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
    tap_hold_interval: 0,
});

const L1_SP: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: 200,
    hold: l(1),
    tap: k(KeyCode::Space),
    config: keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
    tap_hold_interval: 0,
});

const L2_SP: Action<CustomEvent> = Action::HoldTap(&HoldTapAction {
    timeout: 200,
    hold: l(2),
    tap: k(KeyCode::Space),
    config: keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
    tap_hold_interval: 0,
});

pub const NUM_CHORDS: usize = 13;

#[rustfmt::skip]
pub static CHORDS: [ChordDef; NUM_CHORDS] = [
    ((5, 0), &[(0, 5), (0, 6)]), // y + u = bspc
    ((4, 3), &[(0, 6), (0, 7)]), // u + i = del
    ((4, 0), &[(0, 0), (0, 1)]), // q + w = esc
    ((4, 1), &[(2, 1), (2, 2)]), // x + c = M-x
    ((4, 2), &[(2, 2), (2, 3)]), // c + v = spc, grave

    ((4, 4), &[(1, 5), (1, 6)]), // h + j = <
    ((4, 5), &[(1, 6), (1, 7)]), // j + k = :
    ((4, 6), &[(1, 7), (1, 8)]), // k + l = >

    ((4, 7), &[(0, 7), (0, 8)]), // i + o = \
    ((4, 8), &[(0, 8), (0, 9)]), // o + p = /

    ((4, 9), &[(2, 5), (2, 6)]), // n + m = "
    ((4, 10), &[(2, 6), (2, 7)]), // m + , = '
    ((4, 11), &[(2, 7), (2, 8)]), // , + . = _

];

macro_rules! m {
    ($($keys:expr),*) => {
        ::keyberon::action::m(&[$($keys),*].as_slice())
    };
}

#[rustfmt::skip]
pub static LAYERS: Layers  = keyberon::layout::layout! {
    {
        [Q W E R T Y U I O P],
        [A S D F G H J K L ;],
        [Z X C V B N M , . /],
        [n n LGui {ALT_TAB} {L1_SP} {L2_SP} Enter BSpace n n],
        [Escape {m!(KeyCode::LAlt, KeyCode::X)} {m!(KeyCode::Space, KeyCode::Grave)} Delete < {m!(KeyCode::LShift, KeyCode::SColon)} > / '\\' '"'],
        [BSpace '\'' '_' n    n  n n   n      n n],
    }
    {
        [! @ '{' '}' | '`' ~ '\\' n '"' ],
        [# $ '(' ')' n  +  -  /   * '\''],
        [% ^ '[' ']' n  &  =  ,   . '_' ],
        [n n LGui LAlt =  = Tab BSpace n n],
        [n n n    n    n  n n   n      n n],
        [n n n    n    n  n n   n      n n],
    }
    {
        [Kb1 Kb2 Kb3 Kb4 Kb5 Kb6 Kb7 Kb8 Kb9 Kb0],
        [F1  F2  F3  F4  F5  Left Down Up Right VolUp],
        [F6  F7  F8  F9  F10 PgDown {m!(KeyCode::LCtrl, KeyCode::Down)} {m!(KeyCode::LCtrl, KeyCode::Up)} PgUp VolDown],
        [n n F11 F12 t t RAlt End n n],
        [n n n   n   n n n    n   n n],
        [n n n   n   n n n    n   n n],
    }
};
