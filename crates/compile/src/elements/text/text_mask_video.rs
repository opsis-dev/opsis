use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum MaskReveal {
    Left,
    Right,
    Up,
    Down,
    None,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TextMaskVideo {
    #[default = "LAUNCH\nDAY"]
    #[schemars(
        description = "Text",
        extend("x-widget" = "textarea", "x-help" = "The letterforms the media is seen through.")
    )]
    pub text: String,
    #[schemars(
        description = "Media",
        extend(
            "x-widget" = "asset",
            "x-accept" = "video/*,image/*",
            "x-zero-label" = "Gradient",
            "x-help" = "A video or image to show through the letters. Leave empty for the gradient."
        )
    )]
    pub src: String,
    #[default = "linear-gradient(115deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)"]
    #[schemars(
        description = "Gradient fill",
        extend("x-widget" = "gradient", "x-show-if" = { "src": [""] })
    )]
    pub gradient: String,
    #[default(MaskReveal::Left)]
    #[schemars(description = "Reveal", extend("x-help" = "Which way the letters are uncovered."))]
    pub reveal: MaskReveal,
    #[default = 26.0]
    #[schemars(description = "Drift", extend("x-advanced" = true))]
    pub drift: f64,
    #[default = 0.12]
    #[schemars(
        description = "Zoom",
        extend(
            "x-step" = 0.01,
            "x-help" = "Extra scale the media breathes through. 0 holds it still.",
            "x-advanced" = true
        )
    )]
    pub zoom: f64,
    #[default = 30]
    #[schemars(
        description = "Animation length",
        range(min = 1, max = 300),
        extend(
            "x-step" = 1,
            "x-unit" = "frames",
            "x-help" = "Length of the reveal.",
            "x-advanced" = true
        )
    )]
    pub duration_in_frames: u32,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = 800]
    #[schemars(description = "Font weight")]
    pub font_weight: u32,
    #[schemars(description = "Letter spacing")]
    pub letter_spacing: f64,
    #[default = 0.98]
    #[schemars(description = "Line height", extend("x-step" = 0.01))]
    pub line_height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Font size",
        range(min = 12, max = 600),
        extend(
            "x-zero-label" = "Fit",
            "x-unit" = "px",
            "x-help" = "Leave empty to fit the text to the frame.",
            "x-advanced" = true,
            "type" = "number"
        )
    )]
    pub font_size: Option<f64>,
}

pub fn definition() -> ElementMeta {
    meta::<TextMaskVideo>(
        element(
            ElementKind::Component,
            "text-mask-video",
            "Text mask",
            "Big letters with a video, image or gradient showing through them.",
            ElementCategory::Text,
            120,
        )
        .kind_prop("media", "src")
        .look(NodeLook::of("lucide:type-outline")),
    )
}
