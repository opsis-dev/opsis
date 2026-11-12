use std::hash::{Hash, Hasher};

use opsis_types::numbers::truncate;
use serde::{Deserialize, Serialize};

const PX_STEPS: f64 = 64.0;
const ALPHA_STEPS: f64 = 255.0;
const NAN_KEY: i64 = i64::MIN;

fn quantised(value: f64, steps: f64) -> i64 {
    if value.is_nan() {
        return NAN_KEY;
    }
    truncate((value * steps).round().clamp(-9.0e18, 9.0e18))
}

fn exact_bits(value: f64) -> u64 {
    if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        (value + 0.0).to_bits()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Px(pub f64);

impl Hash for Px {
    fn hash<H: Hasher>(&self, state: &mut H) {
        quantised(self.0, PX_STEPS).hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Alpha(pub f64);

impl Hash for Alpha {
    fn hash<H: Hasher>(&self, state: &mut H) {
        quantised(self.0.clamp(0.0, 1.0), ALPHA_STEPS).hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Num(pub f64);

impl Hash for Num {
    fn hash<H: Hasher>(&self, state: &mut H) {
        exact_bits(self.0).hash(state);
    }
}

impl From<f64> for Num {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<f64> for Px {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<f64> for Alpha {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerUnit {
    Cqw,
    Cqh,
    Cqi,
    Cqb,
    Cqmin,
    Cqmax,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Length {
    #[default]
    Auto,
    Px(Px),
    Percent(Num),
    Em(Num),
    Rem(Num),
    Ch(Num),
    Ex(Num),
    Lh(Num),
    Vw(Num),
    Vh(Num),
    Container(ContainerUnit, Num),
    Calc(Box<Calc>),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Calc {
    Sum(Vec<Length>),
    Scaled(Length, Num),
    Min(Vec<Length>),
    Max(Vec<Length>),
    Clamp(Length, Length, Length),
}

impl Length {
    pub const ZERO: Self = Self::Px(Px(0.0));

    pub const fn px(value: f64) -> Self {
        Self::Px(Px(value))
    }

    pub const fn percent(value: f64) -> Self {
        Self::Percent(Num(value))
    }

    pub const fn em(value: f64) -> Self {
        Self::Em(Num(value))
    }

    pub fn sum(terms: impl IntoIterator<Item = Self>) -> Self {
        Self::Calc(Box::new(Calc::Sum(terms.into_iter().collect())))
    }

    #[must_use]
    pub fn scaled(self, by: f64) -> Self {
        Self::Calc(Box::new(Calc::Scaled(self, Num(by))))
    }

    pub fn min(terms: impl IntoIterator<Item = Self>) -> Self {
        Self::Calc(Box::new(Calc::Min(terms.into_iter().collect())))
    }

    pub fn max(terms: impl IntoIterator<Item = Self>) -> Self {
        Self::Calc(Box::new(Calc::Max(terms.into_iter().collect())))
    }

    pub fn clamp(low: Self, preferred: Self, high: Self) -> Self {
        Self::Calc(Box::new(Calc::Clamp(low, preferred, high)))
    }
}

impl From<f64> for Length {
    fn from(value: f64) -> Self {
        Self::px(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Angle {
    #[default]
    Zero,
    Deg(Num),
    Turn(Num),
    Rad(Num),
}

impl Angle {
    pub const fn deg(value: f64) -> Self {
        Self::Deg(Num(value))
    }

    pub fn degrees(&self) -> f64 {
        match self {
            Self::Zero => 0.0,
            Self::Deg(Num(d)) => *d,
            Self::Turn(Num(t)) => t * 360.0,
            Self::Rad(Num(r)) => r.to_degrees(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Edges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Clone> Edges<T> {
    pub fn all(value: T) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    pub fn axes(vertical: T, horizontal: T) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Corners<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_right: T,
    pub bottom_left: T,
}

impl<T: Clone> Corners<T> {
    pub fn all(value: T) -> Self {
        Self {
            top_left: value.clone(),
            top_right: value.clone(),
            bottom_right: value.clone(),
            bottom_left: value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct Position2 {
    pub x: Length,
    pub y: Length,
}

impl Position2 {
    pub const fn center() -> Self {
        Self {
            x: Length::percent(50.0),
            y: Length::percent(50.0),
        }
    }
}
