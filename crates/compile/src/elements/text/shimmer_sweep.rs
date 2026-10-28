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
pub struct ShimmerSweep {
    #[default = "Generating"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = "#3f3f46"]
    #[schemars(
        description = "Base color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub base_color: String,
    #[default = "#fafafa"]
    #[schemars(
        description = "Shine color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub shine_color: String,
    #[default = 96]
    #[schemars(description = "Font size", range(min = 12, max = 200), extend("x-step" = 1))]
    pub font_size: u32,
    #[default(FontWeight::W700)]
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
    meta::<ShimmerSweep>(
        element(
            ElementKind::Component,
            "shimmer-sweep",
            "Shimmer sweep",
            "A band of light sweeps across the text.",
            ElementCategory::Text,
            120,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Shimmer)),
    )
}
