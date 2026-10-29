use opsis_macros::settings;
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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Callout {
    #[setting("Text", default = "Click here")]
    pub text: String,
    #[setting("Target X (0–1)", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Target Y (0–1)", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
    #[setting("Label side", default = Side::Top)]
    pub side: Side,
    #[setting("Label colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Arrow colour", default = "#f43f5e", check = colour)]
    pub accent: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
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
