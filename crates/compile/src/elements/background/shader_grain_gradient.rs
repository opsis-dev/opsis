use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
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
