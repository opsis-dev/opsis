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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Model3d {
    #[schemars(
        description = "Model",
        extend(
            "x-widget" = "asset",
            "x-accept" = ".glb,.gltf",
            "x-help" = "A 3D model file. A model saved as several files keeps them next to it."
        )
    )]
    pub src: String,
    #[default(ModelMotion::Spin)]
    #[schemars(
        description = "Motion",
        extend(
            "x-labels" = {
                "spin": "Turntable",
                "orbit": "Orbit",
                "swing": "Swing",
                "float": "Float",
                "still": "Still",
                "keyframes": "Keyframes"
            }
        )
    )]
    pub motion: ModelMotion,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera",
            "x-show-if" = { "motion": ["keyframes"] }
        )
    )]
    pub camera: CameraPath,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 3), extend("x-step" = 0.05))]
    pub speed: f64,
    #[schemars(
        description = "Animation",
        extend("x-help" = "The name of the model's own animation to play. Leave empty for the first one.")
    )]
    pub animation: String,
    #[default = 1.0]
    #[schemars(description = "Animation speed", range(min = 0.1, max = 4), extend("x-step" = 0.05))]
    pub animation_speed: f64,
    #[default = true]
    #[schemars(description = "Loop the animation")]
    pub r#loop: bool,
    #[default = 1.0]
    #[schemars(description = "Zoom", range(min = 0.3, max = 3), extend("x-step" = 0.05))]
    pub zoom: f64,
    #[default(Lighting::Studio)]
    #[schemars(
        description = "Lighting",
        extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" })
    )]
    pub lighting: Lighting,
    #[default = true]
    #[schemars(description = "Floor shadow")]
    pub shadow: bool,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Model3d>(element(ElementKind::Component, "model-3d",
            "3D model",
            "A 3D model from your project on a turntable, with studio lighting and its own animation.",
            ElementCategory::Scene,
            150).look(NodeLook::of("lucide:cuboid").moves(NodeMotion::Turn)))
}
