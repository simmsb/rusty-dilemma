use embassy_time::{Duration, Instant};
use num::integer::Roots;

pub struct GlideConfig {
    pub coefficient: u8,
    pub interval: Duration,
    pub trigger_px: u8,
}

pub struct Glide {
    pub dx: i8,
    pub dy: i8,
}

struct GlideStatus {
    v0: i32,
    x: i32,
    y: i32,
    z: u16,
    timer: Instant,
    counter: u16,
    dx: i32,
    dy: i32,
}

impl Default for GlideStatus {
    fn default() -> Self {
        Self {
            v0: Default::default(),
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
            timer: Instant::MIN,
            counter: Default::default(),
            dx: Default::default(),
            dy: Default::default(),
        }
    }
}

pub struct GlideContext {
    config: GlideConfig,
    status: GlideStatus,
}

fn saturating_i32_to_i8(v: i32) -> i8 {
    v.clamp(i8::MIN as i32, i8::MAX as i32) as i8
}

impl GlideContext {
    pub fn new(config: GlideConfig) -> Self {
        GlideContext {
            config,
            status: GlideStatus::default(),
        }
    }

    pub fn check(&mut self) -> Option<Glide> {
        if self.status.z != 0
            || (self.status.dx == 0 && self.status.dy == 0)
            || self.status.timer.elapsed() < self.config.interval
        {
            None
        } else {
            self.generate()
        }
    }

    pub fn generate(&mut self) -> Option<Glide> {
        if self.status.v0 == 0 {
            self.status = GlideStatus::default();
            return None;
        }

        self.status.counter += 1;
        let p = self.status.v0 * self.status.counter as i32
            - self.config.coefficient as i32
                * self.status.counter as i32
                * self.status.counter as i32
                / 2;

        let x = p * self.status.dx / self.status.v0;
        let y = p * self.status.dy / self.status.v0;
        let dx = x - self.status.x;
        let dy = y - self.status.y;

        if (-1..1).contains(&dx) && (-1..1).contains(&dy) {
            self.status = GlideStatus::default();
        } else {
            self.status.x = x;
            self.status.y = y;
            self.status.timer = Instant::now();
        }

        Some(Glide {
            dx: saturating_i32_to_i8(dx),
            dy: saturating_i32_to_i8(dy),
        })
    }

    pub fn start(&mut self) -> Option<Glide> {
        let v0 = if self.status.dx == 0 && self.status.dy == 0 {
            0
        } else {
            ((self.status.dx * 256).pow(2) + (self.status.dy * 256).pow(2)).sqrt()
        };

        if v0 < self.config.trigger_px as i32 * 256 {
            self.status = GlideStatus::default();
            return None;
        }

        self.status = GlideStatus {
            v0,
            x: 0,
            y: 0,
            z: 0,
            counter: 0,
            timer: Instant::now(),
            dx: self.status.dx,
            dy: self.status.dy,
        };

        self.generate()
    }

    pub fn update(&mut self, dx: i16, dy: i16, z: u16) {
        self.status.dx = dx as i32;
        self.status.dy = dy as i32;
        self.status.z = z;
    }
}
