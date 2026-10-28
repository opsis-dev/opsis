use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct HandwritingText {
    #[default = "Signed by hand"]
    #[schemars(description = "Text", extend("x-help" = "The line that gets written."))]
    pub text: String,
    #[default = 0.14]
    #[schemars(
        description = "Pen size",
        extend(
            "x-step" = 0.01,
            "x-help" = "Diameter of the nib, in em. 0 hides it.",
            "x-advanced" = true
        )
    )]
    pub pen_size: f64,
    #[default = "#e8b86d"]
    #[schemars(
        description = "Pen color",
        extend(
            "x-help" = "Nib colour.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub pen_color: String,
    #[default = 0.18]
    #[schemars(
        description = "Ink softness",
        extend(
            "x-step" = 0.01,
            "x-help" = "Softness of the ink edge, as a share of one glyph.",
            "x-advanced" = true
        )
    )]
    pub ink_softness: f64,
    #[default = 1.6]
    #[schemars(
        description = "Wobble",
        extend(
            "x-step" = 0.1,
            "x-help" = "Per-character tilt and baseline drift, in degrees. Fixed, not animated.",
            "x-advanced" = true
        )
    )]
    pub wobble: f64,
    #[default = 3]
    #[schemars(
        description = "Stagger",
        extend(
            "x-unit" = "frames",
            "x-help" = "Frames between one character and the next. This is the writing speed.",
            "x-advanced" = true
        )
    )]
    pub stagger_in_frames: u32,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = "handwriting"]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: String,
    #[default = "#f4f4f5"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Font size",
        range(min = 12, max = 600),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 1,
            "x-unit" = "px",
            "x-help" = "Leave empty to size the text to the frame.",
            "type" = "integer"
        )
    )]
    pub font_size: Option<u32>,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
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
