use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{
    PAPER_SCALE, PaperSizing, generated, paper_color, paper_colors, paper_sizing, quads, value,
};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ShaderGodRays {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Intensity", default = 0.8, range(min = 0, max = 2), step = 0.05)]
    pub intensity: f64,
    #[setting("Density", default = 0.3, range(min = 0, max = 1), step = 0.05)]
    pub density: f64,
    #[setting("Bloom", default = 0.4, range(min = 0, max = 1), step = 0.05, advanced)]
    pub bloom: f64,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#5a5a7e".into(), "#8a8a95".into(), "#ffffff".into(), "#3a3a5c".into()],
        length(min = 1, max = 5)
    )]
    pub colors: Vec<Colour>,
    #[setting("Glow colour", default = "#3a3a5c", check = colour, advanced)]
    pub color_bloom: String,
    #[setting("Source height", default = -0.55, range(min = -1.5, max = 1.5), advanced)]
    pub offset_y: f64,
    #[setting("Source across", default = 0.0, range(min = -1.5, max = 1.5), advanced)]
    pub offset_x: f64,
    #[setting("Ray breakup", default = 0.3, range(min = 0, max = 1), advanced)]
    pub spotty: f64,
    #[setting("Source glow", default = 0.4, range(min = 0, max = 1), advanced)]
    pub mid_intensity: f64,
    #[setting("Source glow size", default = 0.2, range(min = 0, max = 1), advanced)]
    pub mid_size: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderGodRays>(
        element(
            ElementKind::Component,
            "shader-god-rays",
            "Shader god rays",
            "Volumetric light rays radiating from an off-screen source.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:sunrise")),
    )
}

const MAX_COLORS: usize = 5;

pub fn draw(props: &ShaderGodRays, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let colors: Vec<String> = props.colors.iter().map(|c| c.0.clone()).collect();
    let mut settings = paper_sizing(PaperSizing {
        scale: 1.0,
        rotation: 0.0,
        offset: (props.offset_x, props.offset_y),
    });
    settings.extend(paper_colors(&colors, MAX_COLORS));
    settings.extend([
        value("time", vec![cx.frame() / cx.fps() * props.speed]),
        quads("colorBack", paper_color(&props.color_back).to_vec()),
        quads("colorBloom", paper_color(&props.color_bloom).to_vec()),
        value("density", vec![props.density]),
        value("spotty", vec![props.spotty]),
        value("midSize", vec![props.mid_size]),
        value("midIntensity", vec![props.mid_intensity]),
        value("intensity", vec![props.intensity]),
        value("bloom", vec![props.bloom]),
    ]);
    Ok(generated("shader-god-rays", settings, PAPER_SCALE))
}
