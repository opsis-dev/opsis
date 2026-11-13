use opsis_macros::settings;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::Node;
use opsis_scene::style::Style;
use opsis_types::looks::NodeLook;

use crate::elements::background::shader_dithering::{Dither, DitherPattern, dither_field};
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::DitherShape;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct DitherDissolve {
    #[setting("Shape", default = DitherShape::Simplex)]
    pub shape: DitherShape,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting("Ink", default = "#8f88ae", check = colour)]
    pub color_front: String,
    #[setting(
        "Speed",
        default = 1.5,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Dot size",
        default = 1.6,
        range(min = 1, max = 8),
        unit = "px",
        advanced
    )]
    pub dot_size: f64,
    #[setting(
        "Dot grid",
        default = DitherPattern::Grid4x4,
        advanced
    )]
    pub pattern: DitherPattern,
}

pub fn definition() -> ElementMeta {
    meta::<DitherDissolve>(
        element(
            ElementKind::Presentation,
            "dither-dissolve",
            "Dither dissolve",
            "A dithering shader transition that dissolves one scene into the next.",
            ElementCategory::Transition,
            40,
        )
        .look(NodeLook::of("lucide:grip")),
    )
}

const DOT_GROWTH: f64 = 1.75;
const PAPER_DEFAULT_SCALE: f64 = 0.6;

fn clamped(p: f64, from: &[f64], to: &[f64]) -> Result<f64, DrawFailure> {
    Ok(interpolate(p, from, to, &Interpolation::clamped())?)
}

pub fn present(props: &DitherDissolve, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = side.progress;
    let child_opacity = if side.entering {
        clamped(p, &[0.58, 0.66], &[0.0, 1.0])?
    } else {
        clamped(p, &[0.28, 0.36], &[1.0, 0.0])?
    };
    let mut layers = vec![filled(
        &Style::default().opacity(child_opacity),
        side.children,
    )];
    if side.entering {
        let dot = clamped(
            p,
            &[0.0, 1.0],
            &[props.dot_size, props.dot_size * DOT_GROWTH],
        )?;
        let field = dither_field(&Dither {
            time: cx.frame() / cx.fps() * props.speed,
            size: dot,
            color_back: &props.color_back,
            color_front: &props.color_front,
            shape: props.shape,
            pattern: props.pattern,
            scale: PAPER_DEFAULT_SCALE,
            rotation: 0.0,
        });
        let envelope = clamped(p, &[0.0, 0.3, 0.65, 1.0], &[0.0, 1.0, 1.0, 0.0])?;
        layers.push(filled(&Style::default().opacity(envelope), vec![field]));
    }
    Ok(filled(&Style::default(), layers))
}
