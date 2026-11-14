use std::f64::consts::PI;

use opsis_types::numbers::exact_eq;

use crate::spring::{REST_THRESHOLD, SpringConfig, SpringError, measure_spring, spring_progress};

const NEWTON_STEPS: u32 = 8;
const BISECTION_STEPS: u32 = 40;
const PRECISION: f64 = 1e-7;
const SPRING_EASING_FPS: f64 = 60.0;
const BACK_OVERSHOOT: f64 = 1.70158;

#[derive(Debug, Clone, PartialEq)]
pub enum Easing {
    Linear,
    Quad,
    Cubic,
    Poly(f64),
    Sin,
    Circle,
    Exp,
    Elastic(f64),
    Back(f64),
    Bounce,
    Bezier(f64, f64, f64, f64),
    In(Box<Self>),
    Out(Box<Self>),
    InOut(Box<Self>),
    Spring { config: SpringConfig, seconds: f64 },
}

impl Easing {
    pub const EASE: Self = Self::Bezier(0.42, 0.0, 1.0, 1.0);

    pub const fn back() -> Self {
        Self::Back(BACK_OVERSHOOT)
    }

    pub const fn elastic() -> Self {
        Self::Elastic(1.0)
    }

    pub fn ease_in(inner: Self) -> Self {
        Self::In(Box::new(inner))
    }

    pub fn ease_out(inner: Self) -> Self {
        Self::Out(Box::new(inner))
    }

    pub fn ease_in_out(inner: Self) -> Self {
        Self::InOut(Box::new(inner))
    }

    pub fn spring(config: SpringConfig) -> Result<Self, SpringError> {
        let resolved = config.checked()?;
        let frames = measure_spring(SPRING_EASING_FPS, &resolved, REST_THRESHOLD)?;
        Ok(Self::Spring {
            config: resolved,
            seconds: f64::from(frames) / SPRING_EASING_FPS,
        })
    }

    #[expect(
        clippy::suboptimal_flops,
        reason = "each step rounds on its own, and the recorded goldens hold what that draws"
    )]
    pub fn at(&self, t: f64) -> f64 {
        match self {
            Self::Linear => t,
            Self::Quad => t * t,
            Self::Cubic => t * t * t,
            Self::Poly(power) => t.powf(*power),
            Self::Sin => 1.0 - ((t * PI) / 2.0).cos(),
            Self::Circle => 1.0 - (1.0 - t * t).sqrt(),
            Self::Exp => 2.0_f64.powf(10.0 * (t - 1.0)),
            Self::Elastic(bounciness) => {
                let swing = bounciness * PI;
                1.0 - ((t * PI) / 2.0).cos().powi(3) * (t * swing).cos()
            }
            Self::Back(overshoot) => t * t * ((overshoot + 1.0) * t - overshoot),
            Self::Bounce => bounce(t),
            Self::Bezier(x1, y1, x2, y2) => bezier(t, *x1, *y1, *x2, *y2),
            Self::In(inner) => inner.at(t),
            Self::Out(inner) => 1.0 - inner.at(1.0 - t),
            Self::InOut(inner) => {
                if t < 0.5 {
                    inner.at(t * 2.0) / 2.0
                } else {
                    1.0 - inner.at((1.0 - t) * 2.0) / 2.0
                }
            }
            Self::Spring { config, seconds } => {
                if t <= 0.0 {
                    0.0
                } else if t >= 1.0 {
                    1.0
                } else {
                    spring_progress(config, t * seconds)
                }
            }
        }
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn bounce(t: f64) -> f64 {
    if t < 1.0 / 2.75 {
        return 7.5625 * t * t;
    }
    if t < 2.0 / 2.75 {
        let u = t - 1.5 / 2.75;
        return 7.5625 * u * u + 0.75;
    }
    if t < 2.5 / 2.75 {
        let u = t - 2.25 / 2.75;
        return 7.5625 * u * u + 0.9375;
    }
    let u = t - 2.625 / 2.75;
    7.5625 * u * u + 0.984_375
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn curve_point(at: f64, first: f64, second: f64) -> f64 {
    let rest = 1.0 - at;
    3.0 * rest * rest * at * first + 3.0 * rest * at * at * second + at * at * at
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn curve_slope(at: f64, first: f64, second: f64) -> f64 {
    let rest = 1.0 - at;
    3.0 * rest * rest * first + 6.0 * rest * at * (second - first) + 3.0 * at * at * (1.0 - second)
}

fn curve_at_x(x: f64, x1: f64, x2: f64) -> f64 {
    let mut at = x;
    for _ in 0..NEWTON_STEPS {
        let slope = curve_slope(at, x1, x2);
        if slope.abs() < PRECISION {
            break;
        }
        let next = at - (curve_point(at, x1, x2) - x) / slope;
        if !(0.0..=1.0).contains(&next) {
            break;
        }
        at = next;
    }
    if (curve_point(at, x1, x2) - x).abs() < PRECISION {
        return at;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    at = x;
    for _ in 0..BISECTION_STEPS {
        if curve_point(at, x1, x2) < x {
            lo = at;
        } else {
            hi = at;
        }
        at = f64::midpoint(lo, hi);
    }
    at
}

fn bezier(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if exact_eq(x1, y1) && exact_eq(x2, y2) {
        return t;
    }
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    curve_point(curve_at_x(t, x1, x2), y1, y2)
}
