use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LineByLineSlide {
    #[setting(
        "Text",
        default = "Record once.\nShare everywhere.",
        widget = "textarea"
    )]
    pub text: String,
    #[setting("Distance", default = 48, range(min = 0, max = 160), step = 1)]
    pub distance: u32,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LineByLineSlide>(
            element(
                ElementKind::Component,
                "line-by-line-slide",
                "Line by line slide",
                "Each line enters from the left with a staggered slide and exits to the right for a flowing paragraph reveal.",
                ElementCategory::Text,
                90,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:text-align-start")),
        )
}
