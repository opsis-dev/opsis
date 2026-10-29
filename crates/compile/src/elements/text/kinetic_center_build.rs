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
    #[setting(
        "Entry offset",
        default = 88,
        range(min = 20, max = 160),
        step = 1,
        advanced
    )]
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
    #[setting(
        "Word gap",
        default = 10,
        range(min = 0, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub word_gap: u32,
    #[setting(
        "Time per word",
        default = 0.4333,
        range(min = 0.1, max = 2),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub word_time: f64,
    #[setting(
        "Entry blur",
        default = 3.5,
        range(min = 0, max = 20),
        unit = "px",
        advanced
    )]
    pub entry_blur: f64,
    #[setting("Letter spacing", default = -0.03, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
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
