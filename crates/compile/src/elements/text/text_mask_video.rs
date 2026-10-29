use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MaskReveal {
    Left,
    Right,
    Up,
    Down,
    None,
}

#[settings(export)]
pub struct TextMaskVideo {
    #[setting(
        "Text",
        default = "LAUNCH\nDAY",
        widget = "textarea",
        help = "The letterforms the media is seen through."
    )]
    pub text: String,
    #[setting(
        "Media",
        widget = "asset",
        accept = "video/*,image/*",
        zero_label = "Gradient",
        help = "A video or image to show through the letters. Leave empty for the gradient."
    )]
    pub src: String,
    #[setting(
        "Gradient fill",
        default = "linear-gradient(115deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)",
        widget = "gradient",
        show_if = { "src": [""] }
    )]
    pub gradient: String,
    #[setting("Reveal", default = MaskReveal::Left, help = "Which way the letters are uncovered.")]
    pub reveal: MaskReveal,
    #[setting("Drift", default = 26.0, advanced)]
    pub drift: f64,
    #[setting(
        "Zoom",
        default = 0.12,
        step = 0.01,
        help = "Extra scale the media breathes through. 0 holds it still.",
        advanced
    )]
    pub zoom: f64,
    #[setting(
        "Animation length",
        default = 30,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of the reveal.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Font weight", default = 800)]
    pub font_weight: u32,
    #[setting("Letter spacing", advanced)]
    pub letter_spacing: f64,
    #[setting("Line height", default = 0.98, step = 0.01, advanced)]
    pub line_height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Font size",
        range(min = 12, max = 600),
        zero_label = "Fit",
        unit = "px",
        help = "Leave empty to fit the text to the frame.",
        "type" = "number"
    )]
    pub font_size: Option<f64>,
    #[setting(
        "Drift speed",
        default = 1.0,
        range(min = 0, max = 4),
        unit = "×",
        advanced
    )]
    pub drift_speed: f64,
    #[setting(
        "Largest fitted size",
        default = 150.0,
        range(min = 40, max = 600),
        unit = "px per 1080 px of width",
        advanced
    )]
    pub fit_max: f64,
    #[setting("Video starts at", default = 0.0, range(min = 0), unit = "s", advanced)]
    pub start_from: f64,
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
