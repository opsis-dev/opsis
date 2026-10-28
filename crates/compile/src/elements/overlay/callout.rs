use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Callout {
    #[default = "Click here"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 0.5]
    #[schemars(description = "Target X (0–1)", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Target Y (0–1)", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub y: f64,
    #[default(Side::Top)]
    #[schemars(description = "Label side")]
    pub side: Side,
    #[default = "#fafafa"]
    #[schemars(
        description = "Label colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#f43f5e"]
    #[schemars(
        description = "Arrow colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent: String,
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
    meta::<Callout>(
        element(
            ElementKind::Component,
            "callout",
            "Callout",
            "A short note that points something out.",
            ElementCategory::Overlay,
            75,
        )
        .look(NodeLook::of("lucide:message-square-quote")),
    )
}
