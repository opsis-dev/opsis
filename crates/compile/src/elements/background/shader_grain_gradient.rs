use opsis_macros::{choice, settings};
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{
    PAPER_SCALE, PaperSizing, generated, paper_color, paper_colors, paper_sizing, quads, value,
};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum GrainGradientShape {
    Corners,
    Wave,
    Dots,
    Truchet,
    Ripple,
    Blob,
    Sphere,
}

#[settings(export)]
pub struct ShaderGrainGradient {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting(
        "Softness",
        default = 0.6,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Intensity", default = 0.2, range(min = 0, max = 1), step = 0.05)]
    pub intensity: f64,
    #[setting("Noise", default = 0.15, range(min = 0, max = 1), step = 0.05)]
    pub noise: f64,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#3a3a52".into(), "#4a4a68".into(), "#5a5a7e".into()],
        length(min = 2, max = 7)
    )]
    pub colors: Vec<Colour>,
    #[setting("Pattern", default = GrainGradientShape::Corners)]
    pub shape: GrainGradientShape,
    #[setting(
        "Scale",
        default = 1.0,
        range(min = 0.1, max = 4),
        step = 0.05,
        help = "How large the pattern is in the frame.",
        advanced
    )]
    pub scale: f64,
    #[setting(
        "Turn",
        range(min = 0, max = 360),
        step = 1,
        unit = "°",
        help = "Turns the pattern around the middle of the frame.",
        advanced
    )]
    pub rotation: u32,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderGrainGradient>(
        element(
            ElementKind::Component,
            "shader-grain-gradient",
            "Shader grain gradient",
            "Grainy multi-stop gradient that drifts slowly with film-grain texture over the blend.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("tabler:noise-reduction")),
    )
}

const MAX_COLORS: usize = 7;

#[derive(Debug, Clone, PartialEq)]
pub struct Grain {
    pub time: f64,
    pub colors: Vec<String>,
    pub color_back: String,
    pub softness: f64,
    pub intensity: f64,
    pub noise: f64,
    pub shape: f64,
    pub scale: f64,
    pub rotation: f64,
    pub offset: (f64, f64),
}

pub const fn shape_code(shape: GrainGradientShape) -> f64 {
    match shape {
        GrainGradientShape::Wave => 1.0,
        GrainGradientShape::Dots => 2.0,
        GrainGradientShape::Truchet => 3.0,
        GrainGradientShape::Corners => 4.0,
        GrainGradientShape::Ripple => 5.0,
        GrainGradientShape::Blob => 6.0,
        GrainGradientShape::Sphere => 7.0,
    }
}

pub fn grain_field(grain: &Grain) -> Node {
    let mut settings = paper_sizing(PaperSizing {
        scale: grain.scale,
        rotation: grain.rotation,
        offset: grain.offset,
    });
    settings.extend(paper_colors(&grain.colors, MAX_COLORS));
    settings.extend([
        value("time", vec![grain.time]),
        quads("colorBack", paper_color(&grain.color_back).to_vec()),
        value("softness", vec![grain.softness]),
        value("intensity", vec![grain.intensity]),
        value("noise", vec![grain.noise]),
        value("shape", vec![grain.shape]),
    ]);
    generated("shader-grain-gradient", settings, PAPER_SCALE)
}

pub fn draw(props: &ShaderGrainGradient, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    Ok(grain_field(&Grain {
        time: cx.frame() / cx.fps() * props.speed,
        colors: props.colors.iter().map(|c| c.0.clone()).collect(),
        color_back: props.color_back.clone(),
        softness: props.softness,
        intensity: props.intensity,
        noise: props.noise,
        shape: shape_code(props.shape),
        scale: props.scale,
        rotation: f64::from(props.rotation),
        offset: (0.0, 0.0),
    }))
}
