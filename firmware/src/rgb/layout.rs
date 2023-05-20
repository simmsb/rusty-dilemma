pub const NUM_LEDS: u16 = 18 + 18;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Switch,
    Underglow,
}

#[derive(Clone, Copy)]
pub struct Light {
    /// relative distance from the bottom left light on the left board (mm)
    pub location: (i16, i16),

    /// matrix position
    pub position: Option<(u8, u8)>,

    pub kind: Kind,

    pub index: u16,
}

struct UnindexedLight {
    pub location: (i16, i16),
    pub position: Option<(u8, u8)>,
    pub kind: Kind,
}

impl UnindexedLight {
    const fn under(location: (i16, i16)) -> Self {
        Self {
            location,
            position: None,
            kind: Kind::Underglow,
        }
    }

    const fn switch(location: (i16, i16), position: (u8, u8)) -> Self {
        Self {
            location,
            position: Some(position),
            kind: Kind::Switch,
        }
    }
}

const fn index_lights<const N: usize>(lights: [UnindexedLight; N]) -> [Light; N] {
    let mut out: [MaybeUninit<Light>; N] = MaybeUninit::uninit_array();

    let mut i = 0;
    while i < N {
        out[i].write(Light {
            location: lights[i].location,
            position: lights[i].position,
            kind: lights[i].kind,
            index: i as u16,
        });

        i += 1;
    }

    unsafe { MaybeUninit::array_assume_init(out) }
}

mod left {
    use super::{index_lights, Light, UnindexedLight};

    /// the top right switch in the left keyboard is offset in the x axis by this much
    const TOP_RIGHT_LED_OFFSET: i16 = 90;

    const fn u(x: i16, y: i16) -> UnindexedLight {
        UnindexedLight::under((TOP_RIGHT_LED_OFFSET - x, y))
    }

    const fn s(x: i16, y: i16, mx: u8, my: u8) -> UnindexedLight {
        UnindexedLight::switch((TOP_RIGHT_LED_OFFSET - x, y), (4 - mx, my))
    }

    // we use the same relative positions as the right side, just flipped and
    // shifted

    pub static LEFT: &[Light] = &index_lights([
        u(0, 70),
        u(25, 75),
        u(60, 80),
        u(70, 60),
        u(85, 45),
        u(85, 20),
        u(85, 0),
        u(70, 15),
        u(60, 15),
        u(40, -5),
        u(25, -5),
        u(10, -10),
        u(-10, -20),
        u(-20, 10),
        u(-25, 20),
        u(-40, 20),
        u(-40, 40),
        u(-40, 60),
        // row 0
        s(0, 55, 0, 0),
        s(20, 59, 1, 0),
        s(40, 63, 2, 0),
        s(60, 59, 3, 0),
        s(80, 45, 4, 0),
        // row 1
        s(80, 45 - 20, 4, 1),
        s(60, 59 - 20, 3, 1),
        s(40, 63 - 20, 2, 1),
        s(20, 59 - 20, 1, 1),
        s(0, 55 - 20, 0, 1),
        // row 2
        s(0, 55 - 40, 0, 2),
        s(20, 59 - 40, 1, 2),
        s(40, 63 - 40, 2, 2),
        s(60, 59 - 40, 3, 2),
        s(80, 45 - 40, 4, 2),
        // thumb row
        s(-10, 0, 4, 3),
        s(10, -5, 3, 3),
        s(-10, -15, 2, 3),
    ]);
}

use core::mem::MaybeUninit;

pub use left::LEFT;

mod right {
    use super::{index_lights, Light, UnindexedLight};

    /// the top left switch in the right keyboard is offset in the x axis by this much
    const RIGHT_LED_OFFSET: i16 = 180;
    const RIGHT_MATRIX_OFFSET: u8 = 5;

    const fn u(x: i16, y: i16) -> UnindexedLight {
        UnindexedLight::under((x + RIGHT_LED_OFFSET, y))
    }

    const fn s(x: i16, y: i16, mx: u8, my: u8) -> UnindexedLight {
        UnindexedLight::switch((x + RIGHT_LED_OFFSET, y), (mx + RIGHT_MATRIX_OFFSET, my))
    }

    pub static RIGHT: &[Light] = &index_lights([
        u(0, 70),
        u(25, 75),
        u(60, 80),
        u(70, 60),
        u(85, 45),
        u(85, 20),
        u(85, 0),
        u(70, 15),
        u(60, 15),
        u(40, -5),
        u(25, -5),
        u(10, -10),
        u(-10, -20),
        u(-20, 10),
        u(-25, 20),
        u(-40, 20),
        u(-40, 40),
        u(-40, 60),
        // row 0
        s(0, 55, 0, 0),
        s(20, 59, 1, 0),
        s(40, 63, 2, 0),
        s(60, 59, 3, 0),
        s(80, 45, 4, 0),
        // row 1
        s(80, 45 - 20, 4, 1),
        s(60, 59 - 20, 3, 1),
        s(40, 63 - 20, 2, 1),
        s(20, 59 - 20, 1, 1),
        s(0, 55 - 20, 0, 1),
        // row 2
        s(0, 55 - 40, 0, 2),
        s(20, 59 - 40, 1, 2),
        s(40, 63 - 40, 2, 2),
        s(60, 59 - 40, 3, 2),
        s(80, 45 - 40, 4, 2),
        // thumb row
        s(-10, 0, 2, 3),
        s(10, -5, 1, 3),
        s(-10, -15, 0, 3),
    ]);
}

pub use right::RIGHT;
