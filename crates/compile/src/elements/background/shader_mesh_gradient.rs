use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
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
