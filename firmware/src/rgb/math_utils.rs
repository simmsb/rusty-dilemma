#![allow(dead_code)]

use cichlid::ColorRGB;
use fixed_macro::fixed;

use fixed::types::{I12F4, I16F16, I4F12, U0F16, U16F16};
use rand::Rng;

use crate::rng::MyRng;

pub(crate) fn wrapping_delta(a: I16F16, b: I16F16, min: I16F16, max: I16F16) -> I16F16 {
    let half_range = (max - min) / fixed!(2: I16F16);

    let d = b.wrapping_sub(a);

    if d.abs() <= half_range {
        d
    } else {
        b.wrapping_sub(max).wrapping_add(min.wrapping_sub(a))
    }
}

pub(crate) fn wrapping_delta_u(a: U16F16, b: U16F16, min: U16F16, max: U16F16) -> U16F16 {
    let half_range = (max - min) / fixed!(2: U16F16);

    let d = b.abs_diff(a);

    if d <= half_range {
        d
    } else {
        half_range - d
    }
}

pub(crate) fn sqr(x: I16F16) -> I16F16 {
    x * x
}

pub(crate) fn rand_decimal() -> I4F12 {
    I4F12::from_bits(MyRng.gen()).frac()
}

pub(crate) fn rand_rainbow() -> ColorRGB {
    rainbow(rand_decimal())
}

pub(crate) fn rainbow(x: I4F12) -> ColorRGB {
    let x = fixed!(0.5: I4F12) - x;

    let r = cordic::sin(I4F12::PI * x);
    let g = cordic::sin(I4F12::PI * (x + fixed!(0.333333: I4F12)));
    let b = cordic::sin(I4F12::PI * (x + fixed!(0.666666: I4F12)));

    let r = (r * r).saturating_lerp(I12F4::ZERO, fixed!(255: I12F4));
    let g = (g * g).saturating_lerp(I12F4::ZERO, fixed!(255: I12F4));
    let b = (b * b).saturating_lerp(I12F4::ZERO, fixed!(255: I12F4));

    let r: u8 = r.int().saturating_to_num();
    let g: u8 = g.int().saturating_to_num();
    let b: u8 = b.int().saturating_to_num();

    ColorRGB { r, g, b }
}

pub(crate) fn ease_fade(pct: U0F16) -> u8 {
    let mix = if pct < fixed!(0.5: U0F16) {
        let pct: I16F16 = pct.to_num();
        2 * pct * pct
    } else {
        let pct: I16F16 = pct.to_num();
        let a = fixed!(-2: I16F16) * pct + fixed!(2: I16F16);
        let b = a * a;
        fixed!(1: I16F16) - b / 2
    };

    mix.lerp(fixed!(0: I16F16), fixed!(255: I16F16))
        .int()
        .saturating_to_num()
}
