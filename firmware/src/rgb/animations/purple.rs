use core::num::Wrapping;

use cichlid::ColorRGB;
use embassy_time::Duration;
use fixed::types::{I16F16, U0F16, U16F16};
use fixed_macro::fixed;

use crate::rgb::animation::Animation;

pub struct Purple {
    tick: I16F16,
    noise: PerlinNoise2D,
}

impl Default for Purple {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            noise: PerlinNoise2D::new(fixed!(255.0: U16F16), 0),
        }
    }
}

impl Animation for Purple {
    type SyncMessage = I16F16;

    fn tick_rate(&self) -> Duration {
        Duration::from_hz(60)
    }

    fn tick(&mut self) {
        self.tick += fixed!(0.01: I16F16);
        self.tick %= I16F16::PI * 2;
    }

    fn render(&self, light: &crate::rgb::layout::Light) -> cichlid::ColorRGB {
        let (dx, dy) = cordic::sin_cos(self.tick);

        let brightness = self.noise.get_noise(
            (I16F16::from_num(light.location.0) + dx * 100) * fixed!(0.02: I16F16),
            (I16F16::from_num(light.location.1) + dy * 100) * fixed!(0.02: I16F16),
        );

        let brightness = brightness.int().saturating_to_num::<i16>();

        let mut c = ColorRGB {
            r: 141,
            g: 70,
            b: 210,
        };
        c.fade_to_black_by(brightness as u8);
        c
    }

    fn restore_from_sync(&mut self, sync: Self::SyncMessage) {
        self.tick = sync;
    }

    fn new_from_sync(sync: Self::SyncMessage) -> Self {
        let mut new = Self::default();
        new.restore_from_sync(sync);
        new
    }
}

#[derive(Copy, Clone)]
pub struct PerlinNoise2D {
    amplitude: U16F16,
    seed: i32,
}

impl PerlinNoise2D {
    pub fn new(amplitude: U16F16, seed: i32) -> Self {
        Self { amplitude, seed }
    }

    pub fn get_noise(&self, x: I16F16, y: I16F16) -> U16F16 {
        (self.amplitude
            * self
                .get_value(
                    x + I16F16::from_num(self.seed),
                    y + I16F16::from_num(self.seed),
                )
                .to_num::<U16F16>())
        .to_num()
    }

    fn noise(x: i16, y: i16) -> u32 {
        let mut x = Wrapping(x as u32);
        let mut y = Wrapping(y as u32);
        let shift = 16usize;
        let prime = Wrapping(0x45d9f3bu32);
        x = ((x >> shift) ^ x) * prime;
        x = ((x >> shift) ^ x) * prime;
        x = (x >> shift) ^ x;
        y = ((y >> shift) ^ y) * prime;
        y = ((y >> shift) ^ y) * prime;
        y = (y >> shift) ^ y;

        x ^= y + Wrapping(0x9e3779b9u32) + (x << 6) + (y >> 2);

        x.0
    }

    fn grad(x: I16F16, y: I16F16, dx: i16, dy: i16) -> I16F16 {
        let x_int: i16 = x.floor().int().to_num();
        let y_int: i16 = y.floor().int().to_num();

        // -1..1
        let x_p: I16F16 = x.frac() - I16F16::from_num(dx);
        let y_p: I16F16 = y.frac() - I16F16::from_num(dy);

        // -2..2
        match Self::noise(x_int + dx, y_int + dy) & 0b11 {
            0b00 => x_p + y_p,
            0b01 => -x_p + y_p,
            0b10 => x_p - y_p,
            0b11 => -x_p - y_p,
            _ => unreachable!(),
        }
    }

    fn quint(x: U0F16) -> I16F16 {
        let x: I16F16 = x.to_num();

        let x = x
            * x
            * x
            * (x * (x * fixed!(6.0: I16F16) - fixed!(15.0: I16F16)) + fixed!(10.0: I16F16));

        x.frac()
    }

    fn get_value(&self, x: I16F16, y: I16F16) -> U0F16 {
        let x_frac: U0F16 = x.frac().to_num();
        let y_frac: U0F16 = y.frac().to_num();

        let g00 = Self::grad(x, y, 0, 0);
        let g01 = Self::grad(x, y, 0, 1);
        let g10 = Self::grad(x, y, 1, 0);
        let g11 = Self::grad(x, y, 1, 1);

        let curve_x = Self::quint(x_frac);
        let curve_y = Self::quint(y_frac);

        const FACTOR: I16F16 = I16F16::SQRT_2;

        let r = curve_x.lerp(curve_y.lerp(g00, g01), curve_y.lerp(g10, g11)) * FACTOR + I16F16::ONE;
        let r = r / 2;
        let r = r.clamp(I16F16::ZERO, U0F16::MAX.to_num());
        r.saturating_to_num()
    }
}
