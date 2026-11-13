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
pub struct ShaderSwirl {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Bands", default = 4, range(min = 1, max = 12), step = 1)]
    pub band_count: u32,
    #[setting("Twist", default = 0.1, range(min = 0, max = 1), step = 0.05)]
    pub twist: f64,
    #[setting(
        "Softness",
        default = 0.2,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#52527a".into(), "#3a3a5c".into(), "#232338".into()],
        length(min = 2, max = 10)
    )]
    pub colors: Vec<Colour>,
    #[setting("Centre size", default = 0.2, range(min = 0, max = 1), advanced)]
    pub center: f64,
    #[setting("Proportion", default = 0.5, range(min = 0, max = 1), advanced)]
    pub proportion: f64,
    #[setting("Wobble", default = 0.2, range(min = 0, max = 1), advanced)]
    pub noise: f64,
    #[setting("Wobble size", default = 0.4, range(min = 0, max = 1), advanced)]
    pub noise_frequency: f64,
    #[setting("Scale", default = 1.0, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderSwirl>(
        element(
            ElementKind::Component,
            "shader-swirl",
            "Shader swirl",
            "Concentric bands swirling around a center point.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("tabler:spiral")),
    )
}

const MAX_COLORS: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub struct Swirl {
    pub time: f64,
    pub colors: Vec<String>,
    pub color_back: String,
    pub band_count: f64,
    pub twist: f64,
    pub center: f64,
    pub proportion: f64,
    pub softness: f64,
    pub noise: f64,
    pub noise_frequency: f64,
    pub scale: f64,
}

pub fn swirl_field(swirl: &Swirl) -> Node {
    let mut settings = paper_sizing(PaperSizing {
        scale: swirl.scale,
        rotation: 0.0,
        offset: (0.0, 0.0),
    });
    settings.extend(paper_colors(&swirl.colors, MAX_COLORS));
    settings.extend([
        value("time", vec![swirl.time]),
        quads("colorBack", paper_color(&swirl.color_back).to_vec()),
        value("bandCount", vec![swirl.band_count]),
        value("twist", vec![swirl.twist]),
        value("center", vec![swirl.center]),
        value("proportion", vec![swirl.proportion]),
        value("softness", vec![swirl.softness]),
        value("noise", vec![swirl.noise]),
        value("noiseFrequency", vec![swirl.noise_frequency]),
    ]);
    generated("shader-swirl", settings, PAPER_SCALE)
}

pub fn draw(props: &ShaderSwirl, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    Ok(swirl_field(&Swirl {
        time: cx.frame() / cx.fps() * props.speed,
        colors: props.colors.iter().map(|c| c.0.clone()).collect(),
        color_back: props.color_back.clone(),
        band_count: f64::from(props.band_count),
        twist: props.twist,
        center: props.center,
        proportion: props.proportion,
        softness: props.softness,
        noise: props.noise,
        noise_frequency: props.noise_frequency,
        scale: props.scale,
    }))
}
