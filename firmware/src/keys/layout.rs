pub fn chorder() -> super::chord::Chorder {
    dilemma_macros::chords!(
        [(0, 0), (0, 1)] => [(4, 0)],
        [(1, 5), (1, 6)] => [(4, 5)],
        [(2, 7), (2, 8)] => [(5, 0)],
        [(0, 5), (0, 6)] => [(4, 1)],
        [(0, 6), (0, 7)] => [(4, 2)],
        [(2, 5), (2, 6)] => [(4, 8)],
        [(0, 8), (0, 9)] => [(4, 4)],
        [(1, 6), (1, 7)] => [(4, 6)],
        [(1, 7), (1, 8)] => [(4, 7)],
        [(0, 7), (0, 8)] => [(4, 3)],
        [(2, 6), (2, 7)] => [(4, 9)],
    )
}
pub static LAYERS: ::keyberon::layout::Layers<10, 6, 3, super::CustomEvent> = [
    [
        [
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Q),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::W),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::E),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::R),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::T),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Y),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::U),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::I),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::O),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::P),
        ],
        [
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LShift),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::A),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::S),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::D),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::G),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::H),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::J),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::K),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::L),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RShift),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::SColon),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
        ],
        [
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LCtrl),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Z),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::X),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::C),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::V),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::B),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::N),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::M),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Comma),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Dot),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RCtrl),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Slash),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
        ],
        [
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LGui),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Tab),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::Layer(1),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Space),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LAlt),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RAlt),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::Layer(2),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Space),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::Custom(super::CustomEvent::MouseScroll),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Enter),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
        ],
        [
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Escape),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::BSpace),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Delete),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Slash),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Bslash),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Comma,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::SColon,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Dot,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Quote,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Quote),
        ],
        [
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Minus,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
        ],
    ],
    [
        [
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb1,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb2,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::LBracket,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::RBracket,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Bslash,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Grave),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Grave,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Bslash),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Quote,
                ]
                .as_slice(),
            ),
        ],
        [
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LShift),
                tap: ::keyberon::action::Action::MultipleKeyCodes(
                    &[
                        ::keyberon::key_code::KeyCode::LShift,
                        ::keyberon::key_code::KeyCode::Kb3,
                    ]
                    .as_slice(),
                ),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb4,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb9,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb0,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Equal,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Minus),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Slash),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb8,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RShift),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Quote),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
        ],
        [
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LCtrl),
                tap: ::keyberon::action::Action::MultipleKeyCodes(
                    &[
                        ::keyberon::key_code::KeyCode::LShift,
                        ::keyberon::key_code::KeyCode::Kb5,
                    ]
                    .as_slice(),
                ),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb6,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LBracket),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RBracket),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LShift,
                    ::keyberon::key_code::KeyCode::Kb7,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Equal),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Comma),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Dot),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RCtrl),
                tap: ::keyberon::action::Action::MultipleKeyCodes(
                    &[
                        ::keyberon::key_code::KeyCode::LShift,
                        ::keyberon::key_code::KeyCode::Minus,
                    ]
                    .as_slice(),
                ),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
        ],
        [
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LAlt),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Space),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Equal),
            ::keyberon::action::Action::NoOp,
        ],
        [
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::Custom(super::CustomEvent::MouseLeft),
            ::keyberon::action::Action::Custom(super::CustomEvent::MouseRight),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
        ],
        [
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
        ],
    ],
    [
        [
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb1),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb2),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb3),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb4),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb5),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb6),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb7),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb8),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb9),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Kb0),
        ],
        [
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LShift),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F1),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F2),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F3),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F4),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F5),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Left),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Down),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Up),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Right),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RShift),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::VolUp),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
        ],
        [
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::LCtrl),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F6),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F7),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F8),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F9),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::F10),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::PgDown),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LCtrl,
                    ::keyberon::key_code::KeyCode::Down,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::MultipleKeyCodes(
                &[
                    ::keyberon::key_code::KeyCode::LCtrl,
                    ::keyberon::key_code::KeyCode::Up,
                ]
                .as_slice(),
            ),
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::PgUp),
            ::keyberon::action::Action::HoldTap(&::keyberon::action::HoldTapAction {
                timeout: 400,
                hold: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::RCtrl),
                tap: ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::VolDown),
                config: ::keyberon::action::HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 200,
            }),
        ],
        [
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::KeyCode(::keyberon::key_code::KeyCode::Equal),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
        ],
        [
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::Custom(super::CustomEvent::MouseLeft),
            ::keyberon::action::Action::Custom(super::CustomEvent::MouseRight),
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
        ],
        [
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
            ::keyberon::action::Action::NoOp,
        ],
    ],
];
