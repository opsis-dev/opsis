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
pub struct TrackingIn {
    #[default = "tracking in"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 0.5]
    #[schemars(
        description = "Start tracking (em)",
        range(min = 0, max = 2),
        extend("x-step" = 0.05)
    )]
    pub start_tracking: f64,
    #[default = 12]
    #[schemars(description = "Start blur", range(min = 0, max = 40), extend("x-step" = 1))]
    pub start_blur: u32,
    #[default = 96]
    #[schemars(description = "Font size", range(min = 12, max = 200), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = "#171717"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
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
    meta::<TrackingIn>(
        element(
            ElementKind::Component,
            "tracking-in",
            "Tracking in",
            "Letter-spacing collapses and blur clears with a springy motion.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Tracking)),
    )
}
