use std::f64::consts::FRAC_PI_2;

use opsis_compile::mix::eq::Biquad;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FilterState {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl FilterState {
    pub fn process(&mut self, filter: &Biquad, input: f64) -> f64 {
        let a0 = filter.a[0];
        let fed = filter.b[2].mul_add(self.x2, filter.b[1].mul_add(self.x1, filter.b[0] * input));
        let back = filter.a[2].mul_add(self.y2, filter.a[1] * self.y1);
        let output = (fed - back) / a0;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Smoother {
    value: f64,
    pull: f64,
}

impl Smoother {
    pub fn new(value: f64, seconds: f64, sample_rate: f64) -> Self {
        Self {
            value,
            pull: 1.0 - (-1.0 / (seconds * sample_rate).max(1.0)).exp(),
        }
    }

    pub fn next(&mut self, target: f64) -> f64 {
        self.value = (target - self.value).mul_add(self.pull, self.value);
        self.value
    }

    pub const fn value(&self) -> f64 {
        self.value
    }
}

pub fn pan_stereo(left: f64, right: f64, pan: f64) -> (f64, f64) {
    let pan = pan.clamp(-1.0, 1.0);
    if pan <= 0.0 {
        let x = (pan + 1.0) * FRAC_PI_2;
        (right.mul_add(x.cos(), left), right * x.sin())
    } else {
        let x = pan * FRAC_PI_2;
        (left * x.cos(), left.mul_add(x.sin(), right))
    }
}

pub fn db_to_gain(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}
