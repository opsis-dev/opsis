use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Lighting, Motion};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Logo3d {
    #[schemars(
        description = "Logo file",
        extend(
            "x-widget" = "asset",
            "x-accept" = ".svg,.png,.jpg,.jpeg,.webp",
            "x-zero-label" = "Sample logo",
            "x-help" = "A drawn logo is given depth; a picture of a logo goes on a card."
        )
    )]
    pub src: String,
    #[default = 0.25]
    #[schemars(description = "Depth", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub depth: f64,
    #[schemars(
        description = "Colour",
        extend("x-widget" = "color", "x-zero-label" = "The logo's own colours")
    )]
    pub color: String,
    #[default = 0.3]
    #[schemars(description = "Metal", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub metalness: f64,
    #[default(Motion::Spin)]
    #[schemars(
        description = "Motion",
        extend(
            "x-labels" = {
                "spin": "Spin",
                "tilt-in": "Tilt in",
                "float": "Float",
                "orbit": "Orbit",
                "swing": "Swing",
                "still": "Still",
                "keyframes": "Keyframes"
            }
        )
    )]
    pub motion: Motion,
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
    #[default(Lighting::Studio)]
    #[schemars(
        description = "Lighting",
        extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" })
    )]
    pub lighting: Lighting,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Logo3d>(
        element(
            ElementKind::Component,
            "logo-3d",
            "3D logo",
            "Your logo in 3D, spinning, tilting in or floating, with studio lighting.",
            ElementCategory::Scene,
            150,
        )
        .look(NodeLook::of("lucide:rotate-3d").moves(NodeMotion::Turn)),
    )
}
