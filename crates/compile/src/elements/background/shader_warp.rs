use opsis_macros::{choice, settings};
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{
    PAPER_SCALE, PaperSizing, generated, paper_colors, paper_sizing, value,
};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum WarpShape {
    #[label("Checks")]
    Checks,
    #[label("Stripes")]
    Stripes,
    #[label("Edge")]
    Edge,
}

#[settings(export)]
pub struct ShaderWarp {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Distortion", default = 0.2, range(min = 0, max = 1), step = 0.05)]
    pub distortion: f64,
    #[setting("Swirl", default = 0.4, range(min = 0, max = 1), step = 0.05)]
    pub swirl: f64,
    #[setting(
        "Softness",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Proportion", default = 0.5, range(min = 0, max = 1), step = 0.05)]
    pub proportion: f64,
    #[setting(
        "Colours",
        default = vec!["#12121a".into(), "#3a3a5c".into(), "#12121a".into(), "#52527a".into()],
        length(min = 2, max = 10)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Pattern",
        default = WarpShape::Checks
    )]
    pub shape: WarpShape,
    #[setting("Pattern size", default = 0.1, range(min = 0, max = 1), advanced)]
    pub shape_scale: f64,
    #[setting(
        "Swirl layers",
        default = 10,
        range(min = 2, max = 20),
        step = 1,
        advanced
    )]
    pub swirl_iterations: u32,
    #[setting("Scale", default = 1.0, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderWarp>(
        element(
            ElementKind::Component,
            "shader-warp",
            "Shader warp",
            "Domain-warped color field that folds and stretches like liquid.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:droplets")),
    )
}

const MAX_COLORS: usize = 10;

const fn shape_code(shape: WarpShape) -> f64 {
    match shape {
        WarpShape::Checks => 0.0,
        WarpShape::Stripes => 1.0,
        WarpShape::Edge => 2.0,
    }
}

pub fn draw(props: &ShaderWarp, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let colors: Vec<String> = props.colors.iter().map(|c| c.0.clone()).collect();
    let mut settings = paper_sizing(PaperSizing {
        scale: props.scale,
        rotation: 0.0,
        offset: (0.0, 0.0),
    });
    settings.extend(paper_colors(&colors, MAX_COLORS));
    settings.extend([
        value("time", vec![cx.frame() / cx.fps() * props.speed]),
        value("proportion", vec![props.proportion]),
        value("softness", vec![props.softness]),
        value("shape", vec![shape_code(props.shape)]),
        value("shapeScale", vec![props.shape_scale]),
        value("distortion", vec![props.distortion]),
        value("swirl", vec![props.swirl]),
        value("swirlIterations", vec![f64::from(props.swirl_iterations)]),
    ]);
    Ok(generated("shader-warp", settings, PAPER_SCALE))
}
