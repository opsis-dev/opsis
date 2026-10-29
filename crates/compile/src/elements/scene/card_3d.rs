use opsis_macros::settings;
use opsis_types::looks::NodeLook;
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
pub struct Card3d {
    #[setting(
        "Image",
        widget = "asset",
        accept = "image/*",
        zero_label = "Sample screen"
    )]
    pub src: String,
    #[setting(
        "Rounded corners",
        default = 0.04,
        range(min = 0, max = 0.2),
        step = 0.01
    )]
    pub corner_radius: f64,
    #[setting("Thickness", default = 0.04, range(min = 0.01, max = 0.3), step = 0.01)]
    pub depth: f64,
    #[setting(
        "Tilt",
        default = 40,
        range(min = 0, max = 80),
        step = 1,
        unit = "°",
        help = "How far the card is turned before it settles."
    )]
    pub tilt: u32,
    #[setting(
        "Motion",
        default = Motion::TiltIn,
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
        default = Lighting::Soft,
        labels = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "transparent")]
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
