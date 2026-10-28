use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct KineticHeadline {
    #[default = "Ship *faster* with demos that *move*"]
    #[schemars(description = "Text — wrap words in *asterisks* to emphasise")]
    pub text: String,
    #[default = "#fafafa"]
    #[schemars(
        description = "Text colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#facc15"]
    #[schemars(
        description = "Emphasis colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub emphasis_color: String,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
    #[default = 104]
    #[schemars(description = "Font size", range(min = 24, max = 240), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = 4]
    #[schemars(
        description = "Time between words",
        range(min = 1, max = 20),
        extend("x-step" = 1, "x-unit" = "frames")
    )]
    pub word_stagger: u32,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<KineticHeadline>(
        element(
            ElementKind::Component,
            "kinetic-headline",
            "Animated headline",
            "A headline that animates in word by word.",
            ElementCategory::Text,
            90,
        )
        .look(NodeLook::of("lucide:whole-word")),
    )
}
