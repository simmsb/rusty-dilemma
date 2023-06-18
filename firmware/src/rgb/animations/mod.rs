use cichlid::ColorRGB;
use embassy_time::Duration;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::rng::MyRng;

use super::{animation::Animation, layout::Light};

pub mod null;
pub mod perlin;

pub enum DynAnimation {
    Perlin(perlin::Perlin),
    Null(null::Null),
}

impl DynAnimation {
    pub fn random() -> Self {
        const OPTS: &[fn() -> DynAnimation] = &[|| DynAnimation::Perlin(perlin::Perlin::default())];
        OPTS.choose(&mut MyRng).unwrap()()
    }
}

macro_rules! dyn_impl {
    ($([$variant:ident, $anim:ty]),+) => {
        impl Animation for DynAnimation {
            type SyncMessage = AnimationSync;

            fn construct_sync(&self) -> Self::SyncMessage {
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

            fn restore_from_sync(&mut self, sync: Self::SyncMessage) {
                #[allow(unreachable_patterns)]
                match (self, sync) {
                    $(
                        (Self::$variant(x), AnimationSync::$variant(s)) => x.restore_from_sync(s)
                    ),+,
                    _ => ()
                }
            }

            fn new_from_sync(sync: Self::SyncMessage) -> Self {
                match sync {
                    $(
                        AnimationSync::$variant(x) => DynAnimation::$variant(<$anim>::new_from_sync(x))
                    ),+
                }
            }
        }


    };
}

dyn_impl!([Perlin, perlin::Perlin], [Null, null::Null]);

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub enum AnimationSync {
    Perlin(<perlin::Perlin as Animation>::SyncMessage),
    Null(<null::Null as Animation>::SyncMessage),
}

trait WrapAnimationSync {
    fn wrap_sync(&self) -> AnimationSync;
}

macro_rules! wrap_sync {
    ($anim:ty, $variant:expr) => {
        impl WrapAnimationSync for $anim {
            fn wrap_sync(&self) -> AnimationSync {
                $variant(self.construct_sync())
            }
        }
    };
}

wrap_sync!(perlin::Perlin, AnimationSync::Perlin);
wrap_sync!(null::Null, AnimationSync::Null);
