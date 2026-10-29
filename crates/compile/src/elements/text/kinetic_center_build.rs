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
pub struct KineticCenterBuild {
    #[setting("Text", default = "Words push left.")]
    pub text: String,
    #[setting("Entry offset", default = 88, range(min = 20, max = 160), step = 1)]
    pub entry_offset: u32,
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
    meta::<KineticCenterBuild>(
            element(
                ElementKind::Component,
                "kinetic-center-build",
                "Kinetic center build",
                "A word appears in the center; each new word enters from the right and pushes the line until the full phrase locks centered.",
                ElementCategory::Text,
                60,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:text-align-center")),
        )
}
