use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum Corner {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LowerThird {
    #[default = "Ada Lovelace"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "Founder, Acme"]
    #[schemars(description = "Subtitle")]
    pub subtitle: String,
    #[default(Corner::BottomLeft)]
    #[schemars(description = "Position")]
    pub position: Corner,
    #[default = "#a78bfa"]
    #[schemars(
        description = "Accent",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent: String,
    #[default = "#fafafa"]
    #[schemars(
        description = "Text colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "rgba(10,10,12,0.72)"]
    #[schemars(
        description = "Panel colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub panel: String,
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
    meta::<LowerThird>(
        element(
            ElementKind::Component,
            "lower-third",
            "Lower third",
            "A name or caption bar in the lower part of the screen.",
            ElementCategory::Overlay,
            120,
        )
        .look(NodeLook::of("lucide:panel-bottom")),
    )
}
