use fixed_macro::fixed;

use fixed::types::{I16F16, U16F16};

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
