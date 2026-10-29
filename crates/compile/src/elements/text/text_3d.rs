use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Lighting, Motion};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Center,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreparedTypeface {
    pub typeface: String,
    pub typeface_italic: bool,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Text3d {
    #[setting("Text", default = "Ship faster", widget = "textarea")]
    pub text: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Empty uses the project's heading font."
    )]
    pub font_family: String,
    #[setting("Font weight", default = 800, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting("Italic")]
    pub italic: bool,
    #[setting("Depth", default = 0.35, range(min = 0.05, max = 1), step = 0.05)]
    pub depth: f64,
    #[setting("Rounded edges", default = 0.4, range(min = 0, max = 1), step = 0.05)]
    pub bevel: f64,
    #[setting("Colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Metal", default = 0.2, range(min = 0, max = 1), step = 0.05)]
    pub metalness: f64,
    #[setting("Alignment", default = TextAlign::Center)]
    pub align: TextAlign,
    #[setting(
        "Line length",
        default = 16,
        range(min = 4, max = 60),
        step = 1,
        unit = "letters",
        help = "Long text wraps onto a new line after about this many letters."
    )]
    pub line_width: u32,
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
        default = Lighting::Studio,
        labels = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "transparent")]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Text3d>(
        element(
            ElementKind::Component,
            "text-3d",
            "3D text",
            "Words in solid 3D letters with rounded edges, in any project font.",
            ElementCategory::Text,
            120,
        )
        .look(NodeLook::of("lucide:case-sensitive").moves(NodeMotion::Turn)),
    )
}
