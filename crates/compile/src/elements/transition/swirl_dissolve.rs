use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::Node;
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::background::shader_swirl::{Swirl, swirl_field};
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SwirlDissolve {
    #[setting("Bands", default = 10, range(min = 1, max = 12), step = 1)]
    pub band_count: u32,
    #[setting(
        "Softness",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#1f1d29".into(), "#413d56".into(), "#8f88ae".into()],
        length(min = 2, max = 10)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Grain", default = 0.2, range(min = 0, max = 1), advanced)]
    pub noise: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SwirlDissolve>(
        element(
            ElementKind::Presentation,
            "swirl-dissolve",
            "Swirl dissolve",
            "A swirl shader transition that unwinds open and winds shut to reveal the next scene.",
            ElementCategory::Transition,
            104,
        )
        .look(NodeLook::of("tabler:spiral")),
    )
}

fn clamped(p: f64, from: &[f64], to: &[f64]) -> Result<f64, DrawFailure> {
    Ok(interpolate(p, from, to, &Interpolation::clamped())?)
}

fn eased(p: f64, from: &[f64], to: &[f64], easing: Easing) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        p,
        from,
        to,
        &Interpolation::clamped().eased(easing),
    )?)
}

fn scaled_blurred(opacity: f64, scale: f64, blur: f64) -> Style {
    Style::default()
        .opacity(opacity)
        .transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))])
        .filter(vec![Filter::Blur(Length::px(blur))])
}

fn last_frame_progress(side: &Transition) -> f64 {
    if side.duration > 1.0 {
        (side.progress * side.duration / (side.duration - 1.0)).min(1.0)
    } else {
        1.0
    }
}

const IN_OUT: Easing = Easing::Bezier(0.42, 0.0, 0.58, 1.0);
const SWIRL_CENTER: f64 = 0.2;
const SWIRL_PROPORTION: f64 = 0.5;
const SWIRL_NOISE_FREQUENCY: f64 = 0.4;

pub fn present(props: &SwirlDissolve, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = last_frame_progress(&side);
    let twist = eased(p, &[0.1, 0.42, 0.64, 1.0], &[1.0, 0.0, 0.0, 1.0], IN_OUT)?;
    let child = if side.entering {
        scaled_blurred(
            clamped(p, &[0.77, 0.87], &[0.0, 1.0])?,
            eased(
                p,
                &[0.77, 1.0],
                &[0.3, 1.0],
                Easing::ease_out(Easing::Cubic),
            )?,
            clamped(p, &[0.77, 0.93], &[10.0, 0.0])?,
        )
    } else {
        Style::default().opacity(clamped(p, &[0.06, 0.13], &[1.0, 0.0])?)
    };
    let mut layers = Vec::new();
    if side.entering {
        layers.push(filled(
            &Style::default(),
            vec![swirl_field(&Swirl {
                time: cx.frame() / cx.fps() * props.speed,
                colors: props.colors.iter().map(|c| c.0.clone()).collect(),
                color_back: props.color_back.clone(),
                band_count: f64::from(props.band_count),
                twist,
                center: SWIRL_CENTER,
                proportion: SWIRL_PROPORTION,
                softness: props.softness,
                noise: props.noise,
                noise_frequency: SWIRL_NOISE_FREQUENCY,
                scale: 1.0,
            })],
        ));
    }
    layers.push(filled(&child, side.children));
    Ok(filled(&Style::default(), layers))
}
