use std::f64::consts::PI;

use opsis_types::numbers::truncate;

use crate::values::Num;

pub const REST_THRESHOLD: f64 = 0.005;
const LONGEST_SECONDS: f64 = 600.0;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpringConfig {
    pub mass: f64,
    pub damping: f64,
    pub stiffness: f64,
    pub overshoot_clamping: bool,
}

pub const DEFAULT_SPRING: SpringConfig = SpringConfig {
    mass: 1.0,
    damping: 10.0,
    stiffness: 100.0,
    overshoot_clamping: false,
};

impl Default for SpringConfig {
    fn default() -> Self {
        DEFAULT_SPRING
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("A spring's mass, damping and stiffness have to be above 0.")]
pub struct SpringError;

impl SpringConfig {
    pub fn checked(self) -> Result<Self, SpringError> {
        if self.damping > 0.0 && self.mass > 0.0 && self.stiffness > 0.0 {
            Ok(self)
        } else {
            Err(SpringError)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Motion {
    omega: f64,
    zeta: f64,
}

fn motion_of(config: &SpringConfig) -> Motion {
    Motion {
        omega: (config.stiffness / config.mass).sqrt(),
        zeta: config.damping / (2.0 * (config.stiffness * config.mass).sqrt()),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn damped_of(Motion { omega, zeta }: Motion) -> f64 {
    omega * (1.0 - zeta * zeta).sqrt()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn distance_left(motion: Motion, seconds: f64) -> f64 {
    let Motion { omega, zeta } = motion;
    if zeta < 1.0 {
        let damped = damped_of(motion);
        return (-zeta * omega * seconds).exp()
            * ((damped * seconds).cos() + ((zeta * omega) / damped) * (damped * seconds).sin());
    }
    (-omega * seconds).exp() * (1.0 + omega * seconds)
}

fn first_arrival(motion: Motion) -> f64 {
    let Motion { omega, zeta } = motion;
    if zeta >= 1.0 {
        return f64::INFINITY;
    }
    let damped = damped_of(motion);
    (PI - (damped / (zeta * omega)).atan()) / damped
}

fn envelope(motion: Motion, seconds: f64) -> f64 {
    let Motion { omega, zeta } = motion;
    if zeta < 1.0 {
        let damped = damped_of(motion);
        return (-zeta * omega * seconds).exp() * 1.0_f64.hypot((zeta * omega) / damped);
    }
    distance_left(motion, seconds).abs()
}

pub fn spring_progress(config: &SpringConfig, seconds: f64) -> f64 {
    if seconds <= 0.0 {
        return 0.0;
    }
    let motion = motion_of(config);
    if config.overshoot_clamping && seconds >= first_arrival(motion) {
        return 1.0;
    }
    1.0 - distance_left(motion, seconds)
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
struct Measured {
    fps: Num,
    mass: Num,
    damping: Num,
    stiffness: Num,
    overshoot_clamping: bool,
    threshold: Num,
}

pub fn measure_spring(fps: f64, config: &SpringConfig, threshold: f64) -> Result<u32, SpringError> {
    measured(Measured {
        fps: Num(fps),
        mass: Num(config.mass),
        damping: Num(config.damping),
        stiffness: Num(config.stiffness),
        overshoot_clamping: config.overshoot_clamping,
        threshold: Num(threshold),
    })
}

#[comemo::memoize]
fn measured(key: Measured) -> Result<u32, SpringError> {
    let config = SpringConfig {
        mass: key.mass.0,
        damping: key.damping.0,
        stiffness: key.stiffness.0,
        overshoot_clamping: key.overshoot_clamping,
    };
    measure_spring_uncached(key.fps.0, &config, key.threshold.0)
}

pub fn measure_spring_uncached(
    fps: f64,
    config: &SpringConfig,
    threshold: f64,
) -> Result<u32, SpringError> {
    let resolved = config.checked()?;
    let motion = motion_of(&resolved);
    let longest: u32 = truncate((LONGEST_SECONDS * fps).ceil());
    let mut last_away = 0;
    for frame in 0..=longest {
        let seconds = f64::from(frame) / fps;
        if (1.0 - spring_progress(&resolved, seconds)).abs() >= threshold {
            last_away = frame;
        } else if envelope(motion, seconds) < threshold {
            return Ok(last_away + 1);
        }
    }
    Ok(longest)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    pub frame: f64,
    pub fps: f64,
    pub config: SpringConfig,
    pub duration_in_frames: Option<f64>,
    pub delay: f64,
    pub from: f64,
    pub to: f64,
}

impl Spring {
    pub const fn at(frame: f64, fps: f64) -> Self {
        Self {
            frame,
            fps,
            config: DEFAULT_SPRING,
            duration_in_frames: None,
            delay: 0.0,
            from: 0.0,
            to: 1.0,
        }
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn spring(options: &Spring) -> Result<f64, SpringError> {
    let resolved = options.config.checked()?;
    let elapsed = options.frame - options.delay;
    if elapsed <= 0.0 {
        return Ok(options.from);
    }
    if let Some(length) = options.duration_in_frames
        && elapsed > length
    {
        return Ok(options.to);
    }
    let stretch = match options.duration_in_frames {
        None => 1.0,
        Some(length) => f64::from(measure_spring(options.fps, &resolved, REST_THRESHOLD)?) / length,
    };
    let progress = spring_progress(&resolved, (elapsed * stretch) / options.fps);
    Ok(options.from + (options.to - options.from) * progress)
}
