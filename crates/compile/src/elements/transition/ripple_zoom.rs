use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::Node;
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::background::shader_grain_gradient::{Grain, grain_field};
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct RippleZoom {
    #[setting("Zoom", default = 4.0, range(min = 1, max = 8), step = 0.1, advanced)]
    pub zoom: f64,
    #[setting("Intensity", default = 0.5, range(min = 0, max = 1), step = 0.05)]
    pub intensity: f64,
    #[setting(
        "Softness",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()],
        length(min = 2, max = 7)
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
    #[setting("Grain", default = 0.5, range(min = 0, max = 1), step = 0.01, advanced)]
    pub noise: f64,
    #[setting("Horizontal centre", default = 0.5, range(min = 0, max = 1), advanced)]
    pub origin_x: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RippleZoom>(
            element(
                ElementKind::Presentation,
                "ripple-zoom",
                "Ripple zoom",
                "A grain-gradient ripple transition that flies the camera through the center of the rings while the next scene approaches from the depth.",
                ElementCategory::Transition,
                88,
            )
                .look(NodeLook::of("tabler:ripple").moves(NodeMotion::Ripple)),
        )
}

const IN_OUT: Easing = Easing::Bezier(0.42, 0.0, 0.58, 1.0);

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

fn scaled(scale: f64) -> Vec<Transform> {
    vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))]
}

const RIPPLE: f64 = 5.0;

pub fn present(props: &RippleZoom, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = side.progress;
    if !side.entering {
        let style = Style::default()
            .opacity(eased(p, &[0.35, 0.55], &[1.0, 0.0], IN_OUT)?)
            .transform(scaled(eased(
                p,
                &[0.3, 0.62],
                &[1.0, 1.6],
                Easing::ease_in(Easing::Cubic),
            )?));
        return Ok(filled(&style, side.children));
    }
    let field_opacity = eased(p, &[0.08, 0.32], &[0.0, 1.0], IN_OUT)?;
    let field_scale = eased(
        p,
        &[0.32, 1.0],
        &[0.2, props.zoom],
        Easing::Bezier(0.7, 0.0, 0.3, 1.0),
    )?;
    let field = grain_field(&Grain {
        time: cx.frame() / cx.fps() * props.speed,
        colors: props.colors.iter().map(|c| c.0.clone()).collect(),
        color_back: props.color_back.clone(),
        softness: props.softness,
        intensity: props.intensity,
        noise: props.noise,
        shape: RIPPLE,
        scale: field_scale,
        rotation: 0.0,
        offset: ((props.origin_x - 0.5) * 2.0, 0.0),
    });
    let child = Style::default()
        .opacity(clamped(p, &[0.45, 0.62], &[0.0, 1.0])?)
        .transform(scaled(eased(
            p,
            &[0.42, 0.97],
            &[0.2, 1.0],
            Easing::Bezier(0.33, 1.0, 0.68, 1.0),
        )?))
        .filter(vec![Filter::Blur(Length::px(clamped(
            p,
            &[0.42, 0.8],
            &[8.0, 0.0],
        )?))]);
    Ok(filled(
        &Style::default(),
        vec![
            filled(&Style::default().opacity(field_opacity), vec![field]),
            filled(&child, side.children),
        ],
    ))
}
