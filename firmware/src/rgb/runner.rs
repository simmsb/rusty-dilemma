use core::array;

use cichlid::ColorRGB;
use embassy_futures::select::{select, select3};
use embassy_rp::peripherals::PIO1;
use embassy_time::{Duration, Instant, Timer};
use fixed::types::{I16F16, U0F16, U16F16, U32F32};
use fixed_macro::fixed;

use crate::{
    interboard,
    messages::{device_to_device::DeviceToDevice, reliable_msg, unreliable_msg},
    side::get_side,
    utils::Ticker,
};

use super::{
    animation::Animation,
    animations,
    driver::Ws2812,
    layout::{self, Light, NUM_LEDS},
    RGB_CMD_CHANNEL,
};

const MAX_LEVEL: u8 = 180;
const COLOUR_CORRECTION: ColorRGB = ColorRGB::new(190, 200, 255);
const FADE_DURATION: Duration = Duration::from_secs(3);

fn ease_fade(pct: U0F16) -> u8 {
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

fn ease_fade_on_time(duration: Duration) -> u8 {
    if duration > FADE_DURATION {
        255
    } else {
        let n = U32F32::saturating_from_num(duration.as_ticks() as u32);
        let d = U32F32::saturating_from_num(FADE_DURATION.as_ticks() as u32);
        ease_fade((n / d).saturating_to_num())
    }
}

struct PerformingAnimation<'a, T> {
    animation: T,
    ticker: Ticker,
    colours: &'a mut [ColorRGB; NUM_LEDS as usize],
    lights: &'static [Light; NUM_LEDS as usize],
}

impl<'a, T: Animation> PerformingAnimation<'a, T> {
    fn new(
        animation: T,
        colours: &'a mut [ColorRGB; NUM_LEDS as usize],
        lights: &'static [Light; NUM_LEDS as usize],
    ) -> Self {
        let ticker = Ticker::every(animation.tick_rate());

        let mut performing_animation = Self {
            animation,
            ticker,
            colours,
            lights,
        };
        performing_animation.render();
        performing_animation
    }

    fn reconstruct_from(&mut self, other: PerformingAnimation<T>) {
        self.animation = other.animation;
        self.ticker = other.ticker;

        self.render();
    }

    async fn step(&mut self) {
        self.ticker.next().await;

        self.render();
    }

    fn render(&mut self) {
        self.animation.tick();

        for (dest, light) in self.colours.iter_mut().zip(self.lights) {
            let mut color = self.animation.render(light);
            color.scale(MAX_LEVEL);

            if light.kind == layout::Kind::Switch {
                color.scale_from_other(COLOUR_CORRECTION);
            }

            *dest = color;
        }
    }
}

#[embassy_executor::task]
pub async fn rgb_runner(mut driver: Ws2812<'static, PIO1, 0, { NUM_LEDS as usize }>) {
    let mut current_colours = [ColorRGB::Black; NUM_LEDS as usize];
    let mut next_colours = [ColorRGB::Black; NUM_LEDS as usize];

    let lights = if get_side().is_left() {
        &layout::LEFT
    } else {
        &layout::RIGHT
    };

    let mut current = PerformingAnimation::new(
        animations::DynAnimation::Null(animations::null::Null),
        &mut current_colours,
        lights,
    );

    let mut next: Option<(Instant, PerformingAnimation<'_, animations::DynAnimation>)> =
        if crate::side::this_side_has_usb() || cfg!(feature = "probe") {
            let animation = PerformingAnimation::new(
                animations::DynAnimation::random(),
                &mut next_colours,
                lights,
            );

            // reporo the animation to the other side
            let cmd = DeviceToDevice::SetAnimation(animation.animation.construct_sync());
            interboard::send_msg(reliable_msg(cmd)).await;

            Some((Instant::now(), animation))
        } else {
            None
        };

    let mut last_sync = Instant::now();
    const SYNC_PERIOD: Duration = Duration::from_secs(10);

    loop {
        let mut errors = [GammaErrorTracker::default(); NUM_LEDS as usize];

        if let Some((_, next)) = next.take_if(|(f, _)| f.elapsed() > FADE_DURATION) {
            current.reconstruct_from(next);
        }

        if crate::side::this_side_has_usb() && last_sync.elapsed() > SYNC_PERIOD {
            last_sync = Instant::now();

            let cmd = DeviceToDevice::SyncAnimation(current.animation.construct_sync());
            interboard::send_msg(unreliable_msg(cmd)).await;
        }

        if let Ok(cmd) = RGB_CMD_CHANNEL.try_recv() {
            match cmd {
                super::Command::SetNextAnimation(a) => {
                    next = Some((
                        Instant::now(),
                        PerformingAnimation::new(
                            animations::DynAnimation::new_from_sync(a),
                            &mut next_colours,
                            lights,
                        ),
                    ));
                }
                super::Command::SyncAnimation(sync) => {
                    current.animation.sync(sync.clone());
                }
            }
        }

        loop {
            if let Some((fade_start, next)) = next.as_mut() {
                match select3(
                    current.step(),
                    next.step(),
                    Timer::after(Duration::from_hz(1000)),
                )
                .await
                {
                    embassy_futures::select::Either3::First(_) => {
                        break;
                    }
                    embassy_futures::select::Either3::Second(_) => {
                        break;
                    }
                    embassy_futures::select::Either3::Third(_) => {
                        let corrected_colours =
                            array::from_fn::<_, { NUM_LEDS as usize }, _>(|i| {
                                let mut a = current.colours[i];
                                let b = next.colours[i];
                                a.blend(b, ease_fade_on_time(fade_start.elapsed()));
                                errors[i].process(a)
                            });

                        driver.write(&corrected_colours).await;
                    }
                }
            } else {
                match select(current.step(), Timer::after(Duration::from_hz(1000))).await {
                    embassy_futures::select::Either::First(_) => {
                        break;
                    }
                    embassy_futures::select::Either::Second(_) => {
                        let corrected_colours =
                            array::from_fn::<_, { NUM_LEDS as usize }, _>(|i| {
                                errors[i].process(current.colours[i])
                            });

                        driver.write(&corrected_colours).await;
                    }
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
struct GammaErrorTracker {
    r: U16F16,
    g: U16F16,
    b: U16F16,
}

impl GammaErrorTracker {
    fn process(&mut self, color: ColorRGB) -> ColorRGB {
        // color.modify_all(|i| GAMMA[i as usize].int().saturating_to_num());
        // return color;

        let r = GAMMA[color.r as usize] + self.r;
        self.r = r.frac();
        let r = r.int().saturating_to_num();

        let g = GAMMA[color.g as usize] + self.g;
        self.g = g.frac();
        let g = g.int().saturating_to_num();

        let b = GAMMA[color.b as usize] + self.b;
        self.b = b.frac();
        let b = b.int().saturating_to_num();

        ColorRGB { r, g, b }
    }
}

//  ",".join([f"fixed!({255 * ((n / 255) ** 1.9):#.3}: U16F16)" for n in range(256)])
const GAMMA: [U16F16; 256] = [
    fixed!(0.00: U16F16),
    fixed!(0.00683: U16F16),
    fixed!(0.0255: U16F16),
    fixed!(0.0550: U16F16),
    fixed!(0.0951: U16F16),
    fixed!(0.145: U16F16),
    fixed!(0.205: U16F16),
    fixed!(0.275: U16F16),
    fixed!(0.355: U16F16),
    fixed!(0.444: U16F16),
    fixed!(0.542: U16F16),
    fixed!(0.650: U16F16),
    fixed!(0.767: U16F16),
    fixed!(0.892: U16F16),
    fixed!(1.03: U16F16),
    fixed!(1.17: U16F16),
    fixed!(1.32: U16F16),
    fixed!(1.49: U16F16),
    fixed!(1.66: U16F16),
    fixed!(1.84: U16F16),
    fixed!(2.02: U16F16),
    fixed!(2.22: U16F16),
    fixed!(2.43: U16F16),
    fixed!(2.64: U16F16),
    fixed!(2.86: U16F16),
    fixed!(3.09: U16F16),
    fixed!(3.33: U16F16),
    fixed!(3.58: U16F16),
    fixed!(3.83: U16F16),
    fixed!(4.10: U16F16),
    fixed!(4.37: U16F16),
    fixed!(4.65: U16F16),
    fixed!(4.94: U16F16),
    fixed!(5.24: U16F16),
    fixed!(5.55: U16F16),
    fixed!(5.86: U16F16),
    fixed!(6.18: U16F16),
    fixed!(6.51: U16F16),
    fixed!(6.85: U16F16),
    fixed!(7.20: U16F16),
    fixed!(7.55: U16F16),
    fixed!(7.91: U16F16),
    fixed!(8.28: U16F16),
    fixed!(8.66: U16F16),
    fixed!(9.05: U16F16),
    fixed!(9.45: U16F16),
    fixed!(9.85: U16F16),
    fixed!(10.3: U16F16),
    fixed!(10.7: U16F16),
    fixed!(11.1: U16F16),
    fixed!(11.5: U16F16),
    fixed!(12.0: U16F16),
    fixed!(12.4: U16F16),
    fixed!(12.9: U16F16),
    fixed!(13.4: U16F16),
    fixed!(13.8: U16F16),
    fixed!(14.3: U16F16),
    fixed!(14.8: U16F16),
    fixed!(15.3: U16F16),
    fixed!(15.8: U16F16),
    fixed!(16.3: U16F16),
    fixed!(16.8: U16F16),
    fixed!(17.4: U16F16),
    fixed!(17.9: U16F16),
    fixed!(18.4: U16F16),
    fixed!(19.0: U16F16),
    fixed!(19.6: U16F16),
    fixed!(20.1: U16F16),
    fixed!(20.7: U16F16),
    fixed!(21.3: U16F16),
    fixed!(21.9: U16F16),
    fixed!(22.5: U16F16),
    fixed!(23.1: U16F16),
    fixed!(23.7: U16F16),
    fixed!(24.3: U16F16),
    fixed!(24.9: U16F16),
    fixed!(25.6: U16F16),
    fixed!(26.2: U16F16),
    fixed!(26.9: U16F16),
    fixed!(27.5: U16F16),
    fixed!(28.2: U16F16),
    fixed!(28.9: U16F16),
    fixed!(29.5: U16F16),
    fixed!(30.2: U16F16),
    fixed!(30.9: U16F16),
    fixed!(31.6: U16F16),
    fixed!(32.3: U16F16),
    fixed!(33.1: U16F16),
    fixed!(33.8: U16F16),
    fixed!(34.5: U16F16),
    fixed!(35.3: U16F16),
    fixed!(36.0: U16F16),
    fixed!(36.8: U16F16),
    fixed!(37.5: U16F16),
    fixed!(38.3: U16F16),
    fixed!(39.1: U16F16),
    fixed!(39.9: U16F16),
    fixed!(40.6: U16F16),
    fixed!(41.4: U16F16),
    fixed!(42.2: U16F16),
    fixed!(43.1: U16F16),
    fixed!(43.9: U16F16),
    fixed!(44.7: U16F16),
    fixed!(45.6: U16F16),
    fixed!(46.4: U16F16),
    fixed!(47.2: U16F16),
    fixed!(48.1: U16F16),
    fixed!(49.0: U16F16),
    fixed!(49.8: U16F16),
    fixed!(50.7: U16F16),
    fixed!(51.6: U16F16),
    fixed!(52.5: U16F16),
    fixed!(53.4: U16F16),
    fixed!(54.3: U16F16),
    fixed!(55.2: U16F16),
    fixed!(56.2: U16F16),
    fixed!(57.1: U16F16),
    fixed!(58.0: U16F16),
    fixed!(59.0: U16F16),
    fixed!(59.9: U16F16),
    fixed!(60.9: U16F16),
    fixed!(61.9: U16F16),
    fixed!(62.8: U16F16),
    fixed!(63.8: U16F16),
    fixed!(64.8: U16F16),
    fixed!(65.8: U16F16),
    fixed!(66.8: U16F16),
    fixed!(67.8: U16F16),
    fixed!(68.8: U16F16),
    fixed!(69.9: U16F16),
    fixed!(70.9: U16F16),
    fixed!(71.9: U16F16),
    fixed!(73.0: U16F16),
    fixed!(74.0: U16F16),
    fixed!(75.1: U16F16),
    fixed!(76.2: U16F16),
    fixed!(77.2: U16F16),
    fixed!(78.3: U16F16),
    fixed!(79.4: U16F16),
    fixed!(80.5: U16F16),
    fixed!(81.6: U16F16),
    fixed!(82.7: U16F16),
    fixed!(83.8: U16F16),
    fixed!(85.0: U16F16),
    fixed!(86.1: U16F16),
    fixed!(87.2: U16F16),
    fixed!(88.4: U16F16),
    fixed!(89.5: U16F16),
    fixed!(90.7: U16F16),
    fixed!(91.9: U16F16),
    fixed!(93.0: U16F16),
    fixed!(94.2: U16F16),
    fixed!(95.4: U16F16),
    fixed!(96.6: U16F16),
    fixed!(97.8: U16F16),
    fixed!(99.0: U16F16),
    fixed!(1.00e+02: U16F16),
    fixed!(1.01e+02: U16F16),
    fixed!(1.03e+02: U16F16),
    fixed!(1.04e+02: U16F16),
    fixed!(1.05e+02: U16F16),
    fixed!(1.06e+02: U16F16),
    fixed!(1.08e+02: U16F16),
    fixed!(1.09e+02: U16F16),
    fixed!(1.10e+02: U16F16),
    fixed!(1.12e+02: U16F16),
    fixed!(1.13e+02: U16F16),
    fixed!(1.14e+02: U16F16),
    fixed!(1.15e+02: U16F16),
    fixed!(1.17e+02: U16F16),
    fixed!(1.18e+02: U16F16),
    fixed!(1.19e+02: U16F16),
    fixed!(1.21e+02: U16F16),
    fixed!(1.22e+02: U16F16),
    fixed!(1.23e+02: U16F16),
    fixed!(1.25e+02: U16F16),
    fixed!(1.26e+02: U16F16),
    fixed!(1.27e+02: U16F16),
    fixed!(1.29e+02: U16F16),
    fixed!(1.30e+02: U16F16),
    fixed!(1.32e+02: U16F16),
    fixed!(1.33e+02: U16F16),
    fixed!(1.34e+02: U16F16),
    fixed!(1.36e+02: U16F16),
    fixed!(1.37e+02: U16F16),
    fixed!(1.39e+02: U16F16),
    fixed!(1.40e+02: U16F16),
    fixed!(1.41e+02: U16F16),
    fixed!(1.43e+02: U16F16),
    fixed!(1.44e+02: U16F16),
    fixed!(1.46e+02: U16F16),
    fixed!(1.47e+02: U16F16),
    fixed!(1.49e+02: U16F16),
    fixed!(1.50e+02: U16F16),
    fixed!(1.52e+02: U16F16),
    fixed!(1.53e+02: U16F16),
    fixed!(1.55e+02: U16F16),
    fixed!(1.56e+02: U16F16),
    fixed!(1.58e+02: U16F16),
    fixed!(1.59e+02: U16F16),
    fixed!(1.61e+02: U16F16),
    fixed!(1.62e+02: U16F16),
    fixed!(1.64e+02: U16F16),
    fixed!(1.65e+02: U16F16),
    fixed!(1.67e+02: U16F16),
    fixed!(1.68e+02: U16F16),
    fixed!(1.70e+02: U16F16),
    fixed!(1.72e+02: U16F16),
    fixed!(1.73e+02: U16F16),
    fixed!(1.75e+02: U16F16),
    fixed!(1.76e+02: U16F16),
    fixed!(1.78e+02: U16F16),
    fixed!(1.80e+02: U16F16),
    fixed!(1.81e+02: U16F16),
    fixed!(1.83e+02: U16F16),
    fixed!(1.84e+02: U16F16),
    fixed!(1.86e+02: U16F16),
    fixed!(1.88e+02: U16F16),
    fixed!(1.89e+02: U16F16),
    fixed!(1.91e+02: U16F16),
    fixed!(1.93e+02: U16F16),
    fixed!(1.94e+02: U16F16),
    fixed!(1.96e+02: U16F16),
    fixed!(1.98e+02: U16F16),
    fixed!(1.99e+02: U16F16),
    fixed!(2.01e+02: U16F16),
    fixed!(2.03e+02: U16F16),
    fixed!(2.04e+02: U16F16),
    fixed!(2.06e+02: U16F16),
    fixed!(2.08e+02: U16F16),
    fixed!(2.10e+02: U16F16),
    fixed!(2.11e+02: U16F16),
    fixed!(2.13e+02: U16F16),
    fixed!(2.15e+02: U16F16),
    fixed!(2.17e+02: U16F16),
    fixed!(2.18e+02: U16F16),
    fixed!(2.20e+02: U16F16),
    fixed!(2.22e+02: U16F16),
    fixed!(2.24e+02: U16F16),
    fixed!(2.25e+02: U16F16),
    fixed!(2.27e+02: U16F16),
    fixed!(2.29e+02: U16F16),
    fixed!(2.31e+02: U16F16),
    fixed!(2.33e+02: U16F16),
    fixed!(2.35e+02: U16F16),
    fixed!(2.36e+02: U16F16),
    fixed!(2.38e+02: U16F16),
    fixed!(2.40e+02: U16F16),
    fixed!(2.42e+02: U16F16),
    fixed!(2.44e+02: U16F16),
    fixed!(2.46e+02: U16F16),
    fixed!(2.47e+02: U16F16),
    fixed!(2.49e+02: U16F16),
    fixed!(2.51e+02: U16F16),
    fixed!(2.53e+02: U16F16),
    fixed!(2.55e+02: U16F16),
];
