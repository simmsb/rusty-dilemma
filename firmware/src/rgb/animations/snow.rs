use core::ops::Range;

use cichlid::ColorRGB;
use embassy_time::Duration;
use fixed::types::{I16F16, U16F16};
use fixed_macro::fixed;
use rand::{rngs::SmallRng, seq::IteratorRandom, Rng, SeedableRng};
use shared::side::KeyboardSide;

use crate::{
    rgb::{
        self,
        animation::Animation,
        layout::NUM_COLS,
        math_utils::{rand_rainbow, wrapping_delta_u},
    },
    rng::{splitmix64, MyRng},
    side::get_side,
};

struct Snowflake {
    weight_bin: u8,
    x: I16F16,
    y: I16F16,
    instant: U16F16,
    colour: ColorRGB,
}

pub struct Snow {
    tick: U16F16,
    rng: SmallRng,
    snowflakes: heapless::Deque<Snowflake, 16>,
    column_weights: [u8; 5],
}

const TICK_RATE: U16F16 = fixed!(0.5: U16F16);
const Y_BOUNDS: Range<i16> = -30..90;

impl Default for Snow {
    fn default() -> Self {
        let seed: u8 = MyRng.gen();

        Self {
            tick: Default::default(),
            rng: SmallRng::seed_from_u64(splitmix64(seed as u64)),
            snowflakes: Default::default(),
            column_weights: Default::default(),
        }
    }
}

fn tick_delta(a: U16F16, b: U16F16) -> U16F16 {
    wrapping_delta_u(a, b, U16F16::ZERO, U16F16::MAX)
}

fn weighted_iter(
    weights: &[u8; NUM_COLS],
    cols: &[i16; NUM_COLS],
) -> impl Iterator<Item = (u8, i16)> {
    use core::iter::repeat_n;

    let total: u8 = weights.iter().sum();

    itertools::chain!(
        repeat_n((0, cols[0]), 1 + (total - weights[0]) as usize),
        repeat_n((1, cols[1]), 1 + (total - weights[1]) as usize),
        repeat_n((2, cols[2]), 1 + (total - weights[2]) as usize),
        repeat_n((3, cols[3]), 1 + (total - weights[3]) as usize),
        repeat_n((4, cols[4]), 1 + (total - weights[4]) as usize),
    )
}

impl Animation for Snow {
    type SyncMessage = ();

    fn tick_rate(&self) -> embassy_time::Duration {
        Duration::from_hz(60)
    }

    fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(TICK_RATE);

        if self
            .snowflakes
            .back()
            .map_or(false, |s| s.y < I16F16::from_num(Y_BOUNDS.start))
        {
            if let Some(s) = self.snowflakes.pop_back() {
                self.column_weights[s.weight_bin as usize] -= 1;
            }
        }

        for flake in &mut self.snowflakes {
            flake.y -= fixed!(0.7: I16F16);
        }

        if !self.snowflakes.is_full()
            && self.snowflakes.front().map_or(true, |s| {
                tick_delta(s.instant, self.tick) > fixed!(15.0: I16F16)
            })
        {
            let Some((weight_bin, x)) = match get_side() {
                KeyboardSide::Left => {
                    weighted_iter(&self.column_weights, &rgb::layout::left::COLUMNS)
                }
                KeyboardSide::Right => {
                    weighted_iter(&self.column_weights, &rgb::layout::right::COLUMNS)
                }
            }
            .choose(&mut self.rng) else {
                return;
            };

            let snowflake = Snowflake {
                weight_bin,
                x: I16F16::from_num(x),
                y: I16F16::from_num(Y_BOUNDS.end),
                instant: self.tick,
                colour: rand_rainbow(),
            };

            if self.snowflakes.push_front(snowflake).is_ok() {
                self.column_weights[weight_bin as usize] += 1;
            }
        }
    }

    fn render(&self, light: &crate::rgb::layout::Light) -> ColorRGB {
        let xx = I16F16::from_num(light.location.0);
        let yy = I16F16::from_num(light.location.1);

        let mut out = ColorRGB::Black;

        for flake in self.snowflakes.iter() {
            let dx = flake.x.dist(xx);
            let dy = flake.y.dist(yy);

            // let distance = dx / fixed!(20.0: I16F16);
            let distance = dx / fixed!(10.0: I16F16) + dy / fixed!(40.0: I16F16);

            let level = fixed!(1.0: I16F16)
                .saturating_sub(distance)
                .clamp(I16F16::ZERO, I16F16::ONE)
                .lerp(I16F16::ZERO, fixed!(255: I16F16))
                .int()
                .saturating_to_num();

            let mut colour = flake.colour;
            colour.scale(level);

            out.r = out.r.saturating_add(colour.r);
            out.g = out.g.saturating_add(colour.g);
            out.b = out.b.saturating_add(colour.b);
        }

        out
    }

    fn construct_sync(&self) -> Self::SyncMessage {
        ()
    }

    fn sync(&mut self, _sync: Self::SyncMessage) {}

    fn new_from_sync(_sync: Self::SyncMessage) -> Self {
        Self::default()
    }
}
