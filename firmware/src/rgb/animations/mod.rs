use cichlid::ColorRGB;
use embassy_rp::clocks::RoscRng;
use embassy_time::Duration;
use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};

use super::{animation::Animation, layout::Light};

pub mod purple;

pub enum DynAnimation {
    Purple(purple::Purple),
}

impl DynAnimation {
    pub fn random() -> Self {
        const OPTS: &[fn() -> DynAnimation] = &[|| DynAnimation::Purple(purple::Purple::default())];
        OPTS.choose(&mut RoscRng).unwrap()()
    }
}

macro_rules! dyn_impl {
    ($($variant:ident),+) => {
        impl Animation for DynAnimation {
            type SyncMessage = AnimationSync;

            fn construct_sync(&self) -> Option<Self::SyncMessage> {
                match self {
                    $(
                        Self::$variant(x) => x.wrap_sync()
                    ),+
                }
            }

            fn tick_rate(&self) -> Duration {
                match self {
                    $(
                        Self::$variant(x) => x.tick_rate()
                    ),+
                }
            }

            fn tick(&mut self) {
                match self {
                    $(
                        Self::$variant(x) => x.tick()
                    ),+
                }
            }

            fn render(&self, light: &Light) -> ColorRGB {
                match self {
                    $(
                        Self::$variant(x) => x.render(light)
                    ),+
                }
            }
        }


    };
}

dyn_impl!(Purple);

#[derive(Serialize, Deserialize)]
pub enum AnimationSync {
    Purple(<purple::Purple as Animation>::SyncMessage),
}

trait WrapAnimationSync {
    fn wrap_sync(&self) -> Option<AnimationSync>;
}

macro_rules! wrap_sync {
    ($anim:ty, $variant:expr) => {
        impl WrapAnimationSync for $anim {
            fn wrap_sync(&self) -> Option<AnimationSync> {
                self.construct_sync().map($variant)
            }
        }
    };
}

wrap_sync!(purple::Purple, AnimationSync::Purple);
