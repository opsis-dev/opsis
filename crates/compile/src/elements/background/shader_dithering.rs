use opsis_macros::{choice, settings};
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{
    PAPER_SCALE, PaperSizing, generated, paper_color, paper_sizing, quads, value,
};
use crate::elements::shared::DitherShape;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum DitherPattern {
    #[serde(rename = "random")]
    #[label("Random")]
    Random,
    #[serde(rename = "2x2")]
    #[label("2 × 2")]
    Grid2x2,
    #[serde(rename = "4x4")]
    #[label("4 × 4")]
    Grid4x4,
    #[serde(rename = "8x8")]
    #[label("8 × 8")]
    Grid8x8,
}

#[settings(export)]
pub struct ShaderDithering {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Dot Size", default = 2, range(min = 1, max = 8), step = 1)]
    pub size: u32,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting("Front", default = "#6a6a85", check = colour)]
    pub color_front: String,
    #[setting("Pattern", default = DitherShape::Wave)]
    pub shape: DitherShape,
    #[setting(
        "Dot grid",
        default = DitherPattern::Grid4x4,
        advanced,
        labels = { "random": "Random", "2x2": "2 × 2", "4x4": "4 × 4", "8x8": "8 × 8" }
    )]
    pub r#type: DitherPattern,
    #[setting("Scale", default = 0.6, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
    #[setting("Turn", default = 0.0, range(min = 0, max = 360), unit = "°", advanced)]
    pub rotation: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderDithering>(
        element(
            ElementKind::Component,
            "shader-dithering",
            "Shader dithering",
            "Ordered-dither shading over an animated gradient field.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:grip")),
    )
}

const fn shape_code(shape: DitherShape) -> f64 {
    match shape {
        DitherShape::Simplex => 1.0,
        DitherShape::Warp => 2.0,
        DitherShape::Dots => 3.0,
        DitherShape::Wave => 4.0,
        DitherShape::Ripple => 5.0,
        DitherShape::Swirl => 6.0,
        DitherShape::Sphere => 7.0,
    }
}

const fn pattern_code(pattern: DitherPattern) -> f64 {
    match pattern {
        DitherPattern::Random => 1.0,
        DitherPattern::Grid2x2 => 2.0,
        DitherPattern::Grid4x4 => 3.0,
        DitherPattern::Grid8x8 => 4.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dither<'a> {
    pub time: f64,
    pub size: f64,
    pub color_back: &'a str,
    pub color_front: &'a str,
    pub shape: DitherShape,
    pub pattern: DitherPattern,
    pub scale: f64,
    pub rotation: f64,
}

pub fn dither_field(d: &Dither<'_>) -> Node {
    let mut settings = paper_sizing(PaperSizing {
        scale: d.scale,
        rotation: d.rotation,
        offset: (0.0, 0.0),
    });
    settings.extend([
        value("time", vec![d.time]),
        value("pxSize", vec![d.size]),
        quads("colorBack", paper_color(d.color_back).to_vec()),
        quads("colorFront", paper_color(d.color_front).to_vec()),
        value("shape", vec![shape_code(d.shape)]),
        value("ditherType", vec![pattern_code(d.pattern)]),
    ]);
    generated("shader-dithering", settings, PAPER_SCALE)
}

pub fn draw(props: &ShaderDithering, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    Ok(dither_field(&Dither {
        time: cx.frame() / cx.fps() * props.speed,
        size: f64::from(props.size),
        color_back: &props.color_back,
        color_front: &props.color_front,
        shape: props.shape,
        pattern: props.r#type,
        scale: props.scale,
        rotation: props.rotation,
    }))
}
