use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{
    PAPER_SCALE, PaperSizing, generated, paper_color, paper_sizing, quads, value,
};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ShaderPerlinNoise {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Proportion", default = 0.35, range(min = 0, max = 1), step = 0.05)]
    pub proportion: f64,
    #[setting(
        "Softness",
        default = 0.1,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting("Front", default = "#6a6a85", check = colour)]
    pub color_front: String,
    #[setting("Detail", default = 1, range(min = 1, max = 8), step = 1, advanced)]
    pub octave_count: u32,
    #[setting("Roughness", default = 1.0, range(min = 0.3, max = 1), advanced)]
    pub persistence: f64,
    #[setting("Detail spacing", default = 1.5, range(min = 1.5, max = 10), advanced)]
    pub lacunarity: f64,
    #[setting("Scale", default = 1.0, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderPerlinNoise>(
        element(
            ElementKind::Component,
            "shader-perlin-noise",
            "Shader perlin noise",
            "Classic Perlin noise clouds drifting across the frame.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:cloudy")),
    )
}

pub fn draw(props: &ShaderPerlinNoise, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let mut settings = paper_sizing(PaperSizing {
        scale: props.scale,
        rotation: 0.0,
        offset: (0.0, 0.0),
    });
    settings.extend([
        value("time", vec![cx.frame() / cx.fps() * props.speed]),
        quads("colorFront", paper_color(&props.color_front).to_vec()),
        quads("colorBack", paper_color(&props.color_back).to_vec()),
        value("proportion", vec![props.proportion]),
        value("softness", vec![props.softness]),
        value("octaveCount", vec![f64::from(props.octave_count)]),
        value("persistence", vec![props.persistence]),
        value("lacunarity", vec![props.lacunarity]),
    ]);
    Ok(generated("shader-perlin-noise", settings, PAPER_SCALE))
}
