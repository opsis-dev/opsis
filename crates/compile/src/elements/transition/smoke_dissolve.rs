use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::Node;
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{SmokeRing, smoke_ring};
use crate::elements::shared::Colour;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SmokeDissolve {
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting("Colours", default = vec!["#8f88ae".into()], length(min = 1, max = 10))]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Smoke detail", default = 3.0, range(min = 0.5, max = 8), advanced)]
    pub detail: f64,
    #[setting("Horizontal centre", default = 0.5, range(min = 0, max = 1), advanced)]
    pub origin_x: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SmokeDissolve>(
            element(
                ElementKind::Presentation,
                "smoke-dissolve",
                "Smoke dissolve",
                "A smoke-ring shader transition that expands the ring outward to reveal the next scene.",
                ElementCategory::Transition,
                104,
            )
                .look(NodeLook::of("lucide:cloud-fog")),
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

pub fn present(props: &SmokeDissolve, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = last_frame_progress(&side);
    let child = if side.entering {
        scaled_blurred(
            clamped(p, &[0.55, 0.68], &[0.0, 1.0])?,
            eased(
                p,
                &[0.55, 1.0],
                &[0.3, 1.0],
                Easing::ease_out(Easing::Cubic),
            )?,
            clamped(p, &[0.55, 0.75], &[10.0, 0.0])?,
        )
    } else {
        Style::default().opacity(clamped(p, &[0.06, 0.13], &[1.0, 0.0])?)
    };
    let mut layers = Vec::new();
    if side.entering {
        layers.push(filled(
            &Style::default(),
            vec![smoke_ring(&SmokeRing {
                time: cx.frame() / cx.fps() * props.speed,
                color_back: props.color_back.clone(),
                colors: props.colors.iter().map(|c| c.0.clone()).collect(),
                radius: clamped(p, &[0.0, 1.0], &[0.0, 1.0])?,
                scale: clamped(p, &[0.0, 1.0], &[1.0, 2.0])?,
                thickness: clamped(p, &[0.0, 1.0], &[1.0, 1.5])?,
                noise_scale: props.detail,
                offset_x: (props.origin_x - 0.5) * 2.0,
            })],
        ));
    }
    layers.push(filled(&child, side.children));
    Ok(filled(&Style::default(), layers))
}
