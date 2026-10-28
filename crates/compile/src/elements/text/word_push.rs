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
pub struct WordPush {
    #[default = "move at your own pace"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 72]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = 10]
    #[schemars(description = "Frames / word", range(min = 3, max = 30), extend("x-step" = 1))]
    pub word_gap: u32,
    #[default = 0.8]
    #[schemars(
        description = "Gap shrink / word",
        range(min = 0.5, max = 1),
        extend("x-step" = 0.05)
    )]
    pub accel: f64,
    #[default = "#171717"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default(FontWeight::W400)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = "white"]
    #[schemars(
        description = "Background",
        extend(
            "x-help" = "Set it to transparent to lay the text over other layers.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub background: String,
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
    meta::<WordPush>(
            element(
                ElementKind::Component,
                "word-push",
                "Word push",
                "A phrase builds from its first word centered: each next word drives in from the right and pushes the line left, easing into the new center.",
                ElementCategory::Text,
                90,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Push)),
        )
}
