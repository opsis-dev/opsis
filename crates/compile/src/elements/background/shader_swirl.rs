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

pub fn draw(props: &ShaderSwirl, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let colors: Vec<String> = props.colors.iter().map(|c| c.0.clone()).collect();
    let mut settings = paper_sizing(PaperSizing {
        scale: props.scale,
        rotation: 0.0,
        offset: (0.0, 0.0),
    });
    settings.extend(paper_colors(&colors, MAX_COLORS));
    settings.extend([
        value("time", vec![cx.frame() / cx.fps() * props.speed]),
        quads("colorBack", paper_color(&props.color_back).to_vec()),
        value("bandCount", vec![f64::from(props.band_count)]),
        value("twist", vec![props.twist]),
        value("center", vec![props.center]),
        value("proportion", vec![props.proportion]),
        value("softness", vec![props.softness]),
        value("noise", vec![props.noise]),
        value("noiseFrequency", vec![props.noise_frequency]),
    ]);
    Ok(generated("shader-swirl", settings, PAPER_SCALE))
}
