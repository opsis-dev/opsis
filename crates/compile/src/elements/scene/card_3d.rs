use opsis_types::looks::NodeLook;
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
pub struct Card3d {
    #[schemars(
        description = "Image",
        extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Sample screen")
    )]
    pub src: String,
    #[default = 0.04]
    #[schemars(description = "Rounded corners", range(min = 0, max = 0.2), extend("x-step" = 0.01))]
    pub corner_radius: f64,
    #[default = 0.04]
    #[schemars(description = "Thickness", range(min = 0.01, max = 0.3), extend("x-step" = 0.01))]
    pub depth: f64,
    #[default = 40]
    #[schemars(
        description = "Tilt",
        range(min = 0, max = 80),
        extend(
            "x-step" = 1,
            "x-unit" = "°",
            "x-help" = "How far the card is turned before it settles."
        )
    )]
    pub tilt: u32,
    #[default(Motion::TiltIn)]
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
    #[default(Lighting::Soft)]
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
    meta::<Card3d>(
        element(
            ElementKind::Component,
            "card-3d",
            "3D card",
            "An image on a card with rounded corners and a soft shadow that tilts into place.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide-lab:copy-image")),
    )
}
