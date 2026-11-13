use opsis_macros::{choice, settings};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::Node;
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::background::shader_grain_gradient::{Grain, grain_field};
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum GrainDissolveShape {
    Wave,
    Dots,
    Truchet,
    Corners,
    Ripple,
    Blob,
    Sphere,
}

#[settings(export)]
pub struct GrainDissolve {
    #[setting("Shape", default = GrainDissolveShape::Blob)]
    pub shape: GrainDissolveShape,
    #[setting("Noise", default = 0.3, range(min = 0, max = 1), step = 0.05, advanced)]
    pub noise: f64,
    #[setting(
        "Zoom",
        default = 2.0,
        range(min = 0.5, max = 4),
        step = 0.25,
        advanced
    )]
    pub zoom: f64,
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
    #[setting(
        "Scene blur",
        default = 10,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub scene_blur: u32,
    #[setting(
        "Rotation",
        default = 0.0,
        range(min = 0, max = 360),
        unit = "°",
        advanced
    )]
    pub rotation: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GrainDissolve>(
            element(
                ElementKind::Presentation,
                "grain-dissolve",
                "Grain dissolve",
                "A grain-gradient shader transition that disintegrates the outgoing scene into soft grainy shapes and condenses the next scene out of the noise.",
                ElementCategory::Transition,
                76,
            )
                .look(NodeLook::of("tabler:grain")),
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

const IN_OUT: Easing = Easing::Bezier(0.42, 0.0, 0.58, 1.0);

const fn shape_code(shape: GrainDissolveShape) -> f64 {
    match shape {
        GrainDissolveShape::Wave => 1.0,
        GrainDissolveShape::Dots => 2.0,
        GrainDissolveShape::Truchet => 3.0,
        GrainDissolveShape::Corners => 4.0,
        GrainDissolveShape::Ripple => 5.0,
        GrainDissolveShape::Blob => 6.0,
        GrainDissolveShape::Sphere => 7.0,
    }
}

pub fn present(props: &GrainDissolve, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = side.progress;
    let blur = f64::from(props.scene_blur);
    if !side.entering {
        let style = Style::default()
            .opacity(clamped(p, &[0.14, 0.34], &[1.0, 0.0])?)
            .filter(vec![Filter::Blur(Length::px(clamped(
                p,
                &[0.06, 0.34],
                &[0.0, blur],
            )?))]);
        return Ok(filled(&style, side.children));
    }
    let field_opacity = eased(p, &[0.0, 0.26], &[0.0, 1.0], IN_OUT)?;
    let intensity = eased(p, &[0.08, 0.92], &[0.0, 1.0], IN_OUT)?;
    let softness = eased(p, &[0.08, 0.92], &[0.0, 1.0], IN_OUT)?;
    let field_scale = props.zoom * clamped(p, &[0.0, 1.0], &[0.9, 1.15])?;
    let field = grain_field(&Grain {
        time: cx.frame() / cx.fps() * props.speed,
        colors: props.colors.iter().map(|c| c.0.clone()).collect(),
        color_back: props.color_back.clone(),
        softness,
        intensity,
        noise: props.noise,
        shape: shape_code(props.shape),
        scale: field_scale,
        rotation: props.rotation,
        offset: (0.0, 0.0),
    });
    let child = scaled_blurred(
        clamped(p, &[0.7, 0.86], &[0.0, 1.0])?,
        eased(
            p,
            &[0.7, 1.0],
            &[1.05, 1.0],
            Easing::ease_out(Easing::Cubic),
        )?,
        clamped(p, &[0.7, 0.92], &[blur, 0.0])?,
    );
    Ok(filled(
        &Style::default(),
        vec![
            filled(&Style::default().opacity(field_opacity), vec![field]),
            filled(&child, side.children),
        ],
    ))
}
