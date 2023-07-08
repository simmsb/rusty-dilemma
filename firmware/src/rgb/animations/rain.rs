use core::ops::Range;

use cichlid::ColorRGB;
use embassy_time::Duration;
use fixed::types::{I16F16, U16F16};
use fixed_macro::fixed;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    rgb::{animation::Animation, math_utils::wrapping_delta_u},
    rng::{splitmix64, MyRng},
};

struct Splash {
    x: I16F16,
    y: I16F16,
    instant: U16F16,
    colour: ColorRGB,
}

pub struct Rain {
    tick: U16F16,
    rng: SmallRng,
    colour: Option<ColorRGB>,
    splashes: heapless::Deque<Splash, 4>,
}

const TICK_RATE: U16F16 = fixed!(0.5: U16F16);
const BOUNDS: (Range<i16>, Range<i16>) = (-20..100, -30..70);

impl Default for Rain {
    fn default() -> Self {
        let seed: u8 = MyRng.gen();

        let colour = if MyRng.gen_bool(0.2) {
            None
        } else {
            Some(cichlid::HSV::new(MyRng.gen(), 255, 255).to_rgb_rainbow())
        };

        Self {
            tick: Default::default(),
            rng: SmallRng::seed_from_u64(splitmix64(seed as u64)),
            splashes: Default::default(),
            colour,
        }
    }
}

fn tick_delta(a: U16F16, b: U16F16) -> U16F16 {
    wrapping_delta_u(a, b, U16F16::ZERO, U16F16::MAX)
}

impl Animation for Rain {
    // just have both halves be separate but sync the colour
    type SyncMessage = Option<ColorRGB>;

    fn tick_rate(&self) -> embassy_time::Duration {
        Duration::from_hz(60)
    }

    fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(TICK_RATE);

        if self.splashes.back().map_or(false, |s| {
            tick_delta(s.instant, self.tick) > fixed!(200.0: I16F16)
        }) {
            let _ = self.splashes.pop_back();
        }

        if self.splashes.front().map_or(true, |s| {
            tick_delta(s.instant, self.tick) > fixed!(70.0: I16F16)
        }) {
            let (x_range, y_range) = BOUNDS;
            let x = self.rng.gen_range(x_range);
            let y = self.rng.gen_range(y_range);

            let splash = Splash {
                x: I16F16::from_num(x),
                y: I16F16::from_num(y),
                instant: self.tick,
                colour: self.colour.unwrap_or_else(|| {
                    cichlid::HSV::new(self.rng.gen(), 255, 255).to_rgb_rainbow()
                }),
            };
            let _ = self.splashes.push_front(splash);
        }
    }

    fn render(&self, light: &crate::rgb::layout::Light) -> cichlid::ColorRGB {
        let xx = I16F16::from_num(light.location.0);
        let yy = I16F16::from_num(light.location.1);

        let xx = if crate::side::get_side().is_right() {
            xx.saturating_sub(fixed!(180: I16F16))
        } else {
            xx
        };

        let mut out = ColorRGB::Black;

        for splash in self.splashes.iter() {
            let dx = splash.x.dist(xx);
            let dy = splash.y.dist(yy);

            let dist = dx * dx + dy * dy;
            let dist = I16F16::from_num(embassy_rp::rom_data::float_funcs::fsqrt(
                dist.to_num::<f32>(),
            ));

            let time_delta = tick_delta(self.tick, splash.instant).saturating_to_num::<I16F16>();

            let delta = time_delta.dist(dist);
            let delta = delta / fixed!(40.0: I16F16);

            let b = I16F16::ONE
                .saturating_sub(delta)
                .clamp(I16F16::ZERO, I16F16::ONE);
            let b = b.saturating_mul(b);
            let b = b.saturating_mul(b);

            let b = if time_delta < fixed!(10.0: I16F16) {
                b * time_delta / fixed!(10.0: I16F16)
            } else {
                b
            };

            let level = b
                .clamp(I16F16::ZERO, I16F16::ONE)
                .lerp(I16F16::ZERO, fixed!(255: I16F16))
                .int()
                .saturating_to_num();

            let mut colour = splash.colour;
            colour.scale(level);

            out.r = out.r.saturating_add(colour.r);
            out.g = out.g.saturating_add(colour.g);
            out.b = out.b.saturating_add(colour.b);
        }

        out
    }

    fn construct_sync(&self) -> Self::SyncMessage {
        self.colour
    }

    fn sync(&mut self, sync: Self::SyncMessage) {
        self.colour = sync;
    }

    fn new_from_sync(sync: Self::SyncMessage) -> Self {
        Self {
            colour: sync,
            ..Self::default()
        }
    }
}
