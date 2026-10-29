use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Lighting;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum ModelMotion {
    Spin,
    Orbit,
    Swing,
    Float,
    Still,
    Keyframes,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
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
        default = ModelMotion::Spin,
        labels = {
            "spin": "Turntable",
            "orbit": "Orbit",
            "swing": "Swing",
            "float": "Float",
            "still": "Still",
            "keyframes": "Keyframes"
        }
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
        help = "The name of the model's own animation to play. Leave empty for the first one."
    )]
    pub animation: String,
    #[setting(
        "Animation speed",
        default = 1.0,
        range(min = 0.1, max = 4),
        step = 0.05
    )]
    pub animation_speed: f64,
    #[setting("Loop the animation", default = true)]
    pub r#loop: bool,
    #[setting("Zoom", default = 1.0, range(min = 0.3, max = 3), step = 0.05)]
    pub zoom: f64,
    #[setting(
        "Lighting",
        default = Lighting::Studio,
        labels = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }
    )]
    pub lighting: Lighting,
    #[setting("Floor shadow", default = true)]
    pub shadow: bool,
    #[setting("Background", default = "transparent")]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Model3d>(element(ElementKind::Component, "model-3d",
            "3D model",
            "A 3D model from your project on a turntable, with studio lighting and its own animation.",
            ElementCategory::Scene,
            150).look(NodeLook::of("lucide:cuboid").moves(NodeMotion::Turn)))
}
