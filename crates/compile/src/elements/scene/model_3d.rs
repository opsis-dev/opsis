use opsis_macros::{choice, settings};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Lighting;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum ModelMotion {
    #[label("Turntable")]
    Spin,
    #[label("Orbit")]
    Orbit,
    #[label("Swing")]
    Swing,
    #[label("Float")]
    Float,
    #[label("Still")]
    Still,
    #[label("Keyframes")]
    Keyframes,
}

#[settings(export)]
pub struct Model3d {
    #[setting(
        "Model",
        widget = "asset",
        accept = ".glb,.gltf",
        help = "A 3D model file. A model saved as several files keeps them next to it."
    )]
    pub src: String,
    #[setting(
        "Motion",
        default = ModelMotion::Spin
    )]
    pub motion: ModelMotion,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        show_if = { "motion": ["keyframes"] }
    )]
    pub camera: CameraPath,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 3), step = 0.05)]
    pub speed: f64,
    #[setting(
        "Animation",
        help = "The name of the model's own animation to play. Leave empty for the first one.",
        advanced
    )]
    pub animation: String,
    #[setting(
        "Animation speed",
        default = 1.0,
        range(min = 0.1, max = 4),
        step = 0.05,
        advanced
    )]
    pub animation_speed: f64,
    #[setting("Loop the animation", default = true, advanced)]
    pub r#loop: bool,
    #[setting("Zoom", default = 1.0, range(min = 0.3, max = 3), step = 0.05)]
    pub zoom: f64,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting("Floor shadow", default = true)]
    pub shadow: bool,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background: String,
    #[setting("Shadow strength", default = 0.55, range(min = 0, max = 1), advanced)]
    pub shadow_opacity: f64,
    #[setting("Grow in", default = true, advanced)]
    pub grow_in: bool,
    #[setting("Motion amount", default = 1.0, range(min = 0, max = 2), advanced)]
    pub motion_amount: f64,
    #[setting(
        "Tilt",
        default = 52,
        range(min = 0, max = 80),
        step = 1,
        unit = "°",
        advanced
    )]
    pub tilt: u32,
    #[setting("End facing the camera", advanced)]
    pub end_facing: bool,
    #[setting("Turn the other way", advanced)]
    pub reverse: bool,
    #[setting(
        "Lens",
        default = 30.0,
        range(min = 10, max = 70),
        unit = "°",
        advanced
    )]
    pub lens: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Model3d>(element(ElementKind::Component, "model-3d",
            "3D model",
            "A 3D model from your project on a turntable, with studio lighting and its own animation.",
            ElementCategory::Scene,
            150).look(NodeLook::of("lucide:cuboid").moves(NodeMotion::Turn)))
}
