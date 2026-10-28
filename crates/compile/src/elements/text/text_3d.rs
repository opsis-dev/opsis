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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Text3d {
    #[default = "Ship faster"]
    #[schemars(description = "Text", extend("x-widget" = "textarea"))]
    pub text: String,
    #[schemars(
        description = "Font",
        extend("x-widget" = "font", "x-help" = "Empty uses the project's heading font.")
    )]
    pub font_family: String,
    #[default = 800]
    #[schemars(description = "Font weight", range(min = 100, max = 900), extend("x-step" = 10))]
    pub font_weight: u32,
    #[schemars(description = "Italic")]
    pub italic: bool,
    #[default = 0.35]
    #[schemars(description = "Depth", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub depth: f64,
    #[default = 0.4]
    #[schemars(description = "Rounded edges", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub bevel: f64,
    #[default = "#fafafa"]
    #[schemars(
        description = "Colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = 0.2]
    #[schemars(description = "Metal", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub metalness: f64,
    #[default(TextAlign::Center)]
    #[schemars(description = "Alignment")]
    pub align: TextAlign,
    #[default = 16]
    #[schemars(
        description = "Line length",
        range(min = 4, max = 60),
        extend(
            "x-step" = 1,
            "x-unit" = "letters",
            "x-help" = "Long text wraps onto a new line after about this many letters."
        )
    )]
    pub line_width: u32,
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
