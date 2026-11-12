use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::spring::{DEFAULT_SPRING, Spring, SpringConfig, spring};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Num};

use crate::elements::draw::DrawFailure;

pub const FADED_OUT_OPACITY: f64 = 0.001;
pub const ENTRANCE_FADE_FRAMES: f64 = 14.0;
const ENTRANCE_SPRING: SpringConfig = SpringConfig {
    damping: 200.0,
    mass: 0.8,
    ..DEFAULT_SPRING
};
const RISE_PX: f64 = 40.0;
const BLUR_PX: f64 = 18.0;
const SCALE_FROM: f64 = 0.86;
const SCALE_GROWTH: f64 = 0.14;
const DRAW_ON_S: f64 = 0.45;
const DEFAULT_EXIT_FRAMES: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entrance {
    Rise,
    Fade,
    Blur,
    Scale,
    None,
}

pub fn is_faded_out(opacity: f64) -> bool {
    opacity <= FADED_OUT_OPACITY
}

pub fn clamped(input: f64, from: [f64; 2], to: [f64; 2]) -> Result<f64, DrawFailure> {
    Ok(interpolate(input, &from, &to, &Interpolation::clamped())?)
}

pub fn clamped_eased(
    input: f64,
    from: [f64; 2],
    to: [f64; 2],
    easing: Easing,
) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        input,
        &from,
        &to,
        &Interpolation::clamped().eased(easing),
    )?)
}

pub fn sprung(frame: f64, fps: f64, config: SpringConfig) -> Result<f64, DrawFailure> {
    Ok(spring(&Spring {
        config,
        ..Spring::at(frame, fps)
    })?)
}

pub fn entrance_style(
    entrance: Entrance,
    frame: f64,
    fps: f64,
    delay: f64,
) -> Result<Style, DrawFailure> {
    let style = Style::default();
    if entrance == Entrance::None {
        return Ok(style);
    }
    let f = frame - delay;
    let s = sprung(f, fps, ENTRANCE_SPRING)?;
    let style = style.opacity(clamped(f, [0.0, ENTRANCE_FADE_FRAMES], [0.0, 1.0])?);
    Ok(match entrance {
        Entrance::Rise => style.transform(vec![Transform::Translate(
            Length::ZERO,
            Length::px((1.0 - s) * RISE_PX),
            Length::ZERO,
        )]),
        Entrance::Blur => style.filter(vec![Filter::Blur(Length::px((1.0 - s) * BLUR_PX))]),
        Entrance::Scale => {
            let scale = s.mul_add(SCALE_GROWTH, SCALE_FROM);
            style.transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))])
        }
        Entrance::Fade | Entrance::None => style,
    })
}

#[derive(Debug, Clone, Default)]
pub struct Fade {
    pub fade_in: f64,
    pub fade_out: f64,
    pub ease_in: Option<Easing>,
    pub ease_out: Option<Easing>,
}

fn eased_or_linear(
    input: f64,
    from: [f64; 2],
    to: [f64; 2],
    easing: Option<&Easing>,
) -> Result<f64, DrawFailure> {
    match easing {
        Some(e) => clamped_eased(input, from, to, e.clone()),
        None => clamped(input, from, to),
    }
}

pub fn fade_window(t: f64, from_t: f64, to_t: f64, fade: &Fade) -> Result<f64, DrawFailure> {
    let fade_in = if fade.fade_in > 0.0 {
        eased_or_linear(
            t,
            [from_t, from_t + fade.fade_in],
            [0.0, 1.0],
            fade.ease_in.as_ref(),
        )?
    } else {
        1.0
    };
    let fade_out = if fade.fade_out > 0.0 {
        eased_or_linear(
            t,
            [to_t - fade.fade_out, to_t],
            [1.0, 0.0],
            fade.ease_out.as_ref(),
        )?
    } else {
        1.0
    };
    Ok(fade_in.min(fade_out))
}

pub fn draw_on(t: f64, from_t: f64, length: Option<f64>) -> Result<f64, DrawFailure> {
    clamped_eased(
        t,
        [from_t, from_t + length.unwrap_or(DRAW_ON_S)],
        [0.0, 1.0],
        Easing::ease_in_out(Easing::Cubic),
    )
}

pub fn zoom_over(frame: f64, frames: f64, to: f64) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        &[0.0, frames],
        &[1.0, to],
        &Interpolation::clamp_right(),
    )?)
}

pub fn exit_opacity(frame: f64, duration: f64, frames: Option<f64>) -> Result<f64, DrawFailure> {
    let frames = frames.unwrap_or(DEFAULT_EXIT_FRAMES).max(1.0);
    clamped_eased(
        frame,
        [duration - frames, duration],
        [1.0, 0.0],
        Easing::ease_in(Easing::Quad),
    )
}
