use std::cmp::Ordering;

use crate::easing::Easing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Extrapolate {
    Clamp,
    #[default]
    Extend,
    Identity,
    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Output {
    #[default]
    Linear,
    PerceptualScale,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Interpolation {
    pub easing: Option<Easing>,
    pub left: Extrapolate,
    pub right: Extrapolate,
    pub output: Output,
}

impl Interpolation {
    pub fn clamped() -> Self {
        Self {
            left: Extrapolate::Clamp,
            right: Extrapolate::Clamp,
            ..Self::default()
        }
    }

    pub fn clamp_right() -> Self {
        Self {
            right: Extrapolate::Clamp,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn eased(self, easing: Easing) -> Self {
        Self {
            easing: Some(easing),
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InterpolateError {
    #[error("interpolate needs two ranges of the same length, at least 2 long.")]
    Lengths,
    #[error("interpolate's input range has to rise strictly.")]
    NotRising,
}

fn check_ranges(input: &[f64], output: &[f64]) -> Result<(), InterpolateError> {
    if input.len() < 2 || input.len() != output.len() {
        return Err(InterpolateError::Lengths);
    }
    if input
        .windows(2)
        .any(|w| w[1].partial_cmp(&w[0]) != Some(Ordering::Greater))
    {
        return Err(InterpolateError::NotRising);
    }
    Ok(())
}

fn signed_area(size: f64) -> f64 {
    size.signum_or_zero() * size * size
}

fn size_of_area(area: f64) -> f64 {
    area.signum_or_zero() * area.abs().sqrt()
}

trait JsSign {
    fn signum_or_zero(self) -> f64;
}

impl JsSign for f64 {
    fn signum_or_zero(self) -> f64 {
        if self > 0.0 {
            1.0
        } else if self < 0.0 {
            -1.0
        } else {
            self
        }
    }
}

fn wrapped(input: f64, first: f64, last: f64) -> f64 {
    let span = last - first;
    first + (((input - first) % span) + span) % span
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
pub fn interpolate(
    input: f64,
    input_range: &[f64],
    output_range: &[f64],
    options: &Interpolation,
) -> Result<f64, InterpolateError> {
    check_ranges(input_range, output_range)?;
    let last = input_range.len() - 1;
    let first = input_range[0];
    let end = input_range[last];
    let mut x = input;
    if x < first {
        match options.left {
            Extrapolate::Clamp => return Ok(output_range[0]),
            Extrapolate::Identity => return Ok(input),
            Extrapolate::Wrap => x = wrapped(x, first, end),
            Extrapolate::Extend => {}
        }
    }
    if x > end {
        match options.right {
            Extrapolate::Clamp => return Ok(output_range[last]),
            Extrapolate::Identity => return Ok(input),
            Extrapolate::Wrap => x = wrapped(x, first, end),
            Extrapolate::Extend => {}
        }
    }
    let mut segment = 1;
    while segment < last && x > input_range[segment] {
        segment += 1;
    }
    let (a, b) = (input_range[segment - 1], input_range[segment]);
    let (from, to) = (output_range[segment - 1], output_range[segment]);
    let t = (x - a) / (b - a);
    let eased = options.easing.as_ref().map_or(t, |easing| easing.at(t));
    if options.output == Output::PerceptualScale {
        let (from_area, to_area) = (signed_area(from), signed_area(to));
        return Ok(size_of_area(from_area + (to_area - from_area) * eased));
    }
    Ok(from + (to - from) * eased)
}
