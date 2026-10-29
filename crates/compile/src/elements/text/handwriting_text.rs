use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct HandwritingText {
    #[setting(
        "Text",
        default = "Signed by hand",
        help = "The line that gets written."
    )]
    pub text: String,
    #[setting(
        "Pen size",
        default = 0.14,
        step = 0.01,
        help = "Diameter of the nib, in em. 0 hides it.",
        advanced
    )]
    pub pen_size: f64,
    #[setting("Pen color", default = "#e8b86d", help = "Nib colour.", check = colour)]
    pub pen_color: String,
    #[setting(
        "Ink softness",
        default = 0.18,
        step = 0.01,
        help = "Softness of the ink edge, as a share of one glyph.",
        advanced
    )]
    pub ink_softness: f64,
    #[setting(
        "Wobble",
        default = 1.6,
        step = 0.1,
        help = "Per-character tilt and baseline drift, in degrees. Fixed, not animated.",
        advanced
    )]
    pub wobble: f64,
    #[setting(
        "Stagger",
        default = 3,
        unit = "frames",
        help = "Frames between one character and the next. This is the writing speed.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Variation", default = "handwriting", advanced)]
    pub seed: String,
    #[setting("Color", default = "#f4f4f5", check = colour)]
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Font size",
        range(min = 12, max = 600),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        help = "Leave empty to size the text to the frame.",
        "type" = "integer"
    )]
    pub font_size: Option<u32>,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
}

pub fn definition() -> ElementMeta {
    meta::<HandwritingText>(
        element(
            ElementKind::Component,
            "handwriting-text",
            "Handwriting",
            "Writes the text out by hand, stroke by stroke.",
            ElementCategory::Text,
            120,
        )
        .look(NodeLook::of("lucide:signature").moves(NodeMotion::Draw)),
    )
}
