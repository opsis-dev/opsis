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
#[serde(rename_all = "kebab-case")]
pub enum Corner {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LowerThird {
    #[setting("Title", default = "Ada Lovelace")]
    pub title: String,
    #[setting("Subtitle", default = "Founder, Acme")]
    pub subtitle: String,
    #[setting("Position", default = Corner::BottomLeft)]
    pub position: Corner,
    #[setting("Accent", default = "#a78bfa", check = colour)]
    pub accent: String,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Panel colour", default = "rgba(10,10,12,0.72)", check = colour)]
    pub panel: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
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
