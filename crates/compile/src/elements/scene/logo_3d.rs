use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Lighting, Motion};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Logo3d {
    #[setting(
        "Logo file",
        widget = "asset",
        accept = ".svg,.png,.jpg,.jpeg,.webp",
        zero_label = "Sample logo",
        help = "A drawn logo is given depth; a picture of a logo goes on a card."
    )]
    pub src: String,
    #[setting("Depth", default = 0.25, range(min = 0.05, max = 1), step = 0.05)]
    pub depth: f64,
    #[setting("Colour", widget = "color", zero_label = "The logo's own colours")]
    pub color: String,
    #[setting("Metal", default = 0.3, range(min = 0, max = 1), step = 0.05)]
    pub metalness: f64,
    #[setting(
        "Motion",
        default = Motion::Spin,
        labels = {
            "spin": "Spin",
            "tilt-in": "Tilt in",
            "float": "Float",
            "orbit": "Orbit",
            "swing": "Swing",
            "still": "Still",
            "keyframes": "Keyframes"
        }
    )]
    pub motion: Motion,
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
        "Lighting",
        default = Lighting::Studio,
        labels = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "transparent")]
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
