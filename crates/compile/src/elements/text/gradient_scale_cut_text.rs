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
pub struct GradientScaleCutText {
    #[default = "Introducing"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 520]
    #[schemars(description = "Giant font size", range(min = 160, max = 720), extend("x-step" = 4))]
    pub giant_font_size: u32,
    #[default = 132]
    #[schemars(description = "Compact font size", range(min = 40, max = 240), extend("x-step" = 2))]
    pub compact_font_size: u32,
    #[default(-16)]
    #[schemars(
        description = "Final horizontal anchor offset",
        range(min = -400, max = 400),
        extend("x-step" = 4)
    )]
    pub anchor_offset_x: i32,
    #[default = 400]
    #[schemars(
        description = "Giant phase left travel",
        range(min = 0, max = 800),
        extend("x-step" = 4)
    )]
    pub giant_travel: u32,
    #[default = 160]
    #[schemars(
        description = "Compact entrance travel",
        range(min = 0, max = 400),
        extend("x-step" = 4)
    )]
    pub settle_travel: u32,
    #[default = 0.4333]
    #[schemars(
        description = "Hard cut at",
        range(min = 0.1333, max = 0.8),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub cut_frame: f64,
    #[default = 14]
    #[schemars(
        description = "Reveal edge softness",
        range(min = 0, max = 40),
        extend("x-step" = 1)
    )]
    pub reveal_softness: u32,
    #[default = 12]
    #[schemars(description = "Blur after the cut", range(min = 0, max = 40), extend("x-step" = 1))]
    pub compact_blur: u32,
    #[default = "#f04a14"]
    #[schemars(
        description = "Gradient start",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub gradient_start: String,
    #[default = "#f3f1f1"]
    #[schemars(
        description = "Gradient end",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub gradient_end: String,
    #[default = "#170b0a"]
    #[schemars(
        description = "Unrevealed text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ghost_color: String,
    #[default = "#000000"]
    #[schemars(
        description = "Background color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background_color: String,
    #[default(FontWeight::W700)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GradientScaleCutText>(
            element(
                ElementKind::Component,
                "gradient-scale-cut-text",
                "Gradient scale cut text",
                "An oversized gradient reveal snaps to a separate compact blurred line that settles left while the reveal continues.",
                ElementCategory::Text,
                36,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Snap)),
        )
}
