use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{
    PAPER_SCALE, PaperSizing, generated, paper_colors, paper_sizing, value,
};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ShaderMeshGradient {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Distortion", default = 0.6, range(min = 0, max = 1.5), step = 0.05)]
    pub distortion: f64,
    #[setting("Swirl", default = 0.1, range(min = 0, max = 1), step = 0.05)]
    pub swirl: f64,
    #[setting(
        "Colours",
        default = vec!["#12121a".into(), "#232338".into(), "#3a3a5c".into(), "#52527a".into()],
        length(min = 2, max = 10)
    )]
    pub colors: Vec<Colour>,
    #[setting("Grain", default = 0.0, range(min = 0, max = 1))]
    pub grain_overlay: f64,
    #[setting("Grainy edges", default = 0.0, range(min = 0, max = 1), advanced)]
    pub grain_mixer: f64,
    #[setting("Scale", default = 1.0, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderMeshGradient>(
        element(
            ElementKind::Component,
            "shader-mesh-gradient",
            "Shader mesh gradient",
            "Soft colour blobs that drift and blend into each other.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:blend")),
    )
}

const MAX_COLORS: usize = 10;

pub fn draw(props: &ShaderMeshGradient, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let colors: Vec<String> = props.colors.iter().map(|c| c.0.clone()).collect();
    let mut settings = paper_sizing(PaperSizing {
        scale: props.scale,
        rotation: 0.0,
        offset: (0.0, 0.0),
    });
    settings.extend(paper_colors(&colors, MAX_COLORS));
    settings.extend([
        value("time", vec![cx.frame() / cx.fps() * props.speed]),
        value("distortion", vec![props.distortion]),
        value("swirl", vec![props.swirl]),
        value("grainMixer", vec![props.grain_mixer]),
        value("grainOverlay", vec![props.grain_overlay]),
    ]);
    Ok(generated("shader-mesh-gradient", settings, PAPER_SCALE))
}
