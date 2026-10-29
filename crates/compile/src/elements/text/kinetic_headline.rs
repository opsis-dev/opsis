use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct KineticHeadline {
    #[setting(
        "Text — wrap words in *asterisks* to emphasise",
        default = "Ship *faster* with demos that *move*"
    )]
    pub text: String,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Emphasis colour", default = "#facc15", check = colour)]
    pub emphasis_color: String,
    #[setting("Background", default = "transparent")]
    pub background: String,
    #[setting("Font size", default = 104, range(min = 24, max = 240), step = 1)]
    pub font_size: u32,
    #[setting(
        "Time between words",
        default = 4,
        range(min = 1, max = 20),
        step = 1,
        unit = "frames"
    )]
    pub word_stagger: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Font weight", default = 780, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting("Letter spacing", default = -0.04, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[setting("Line height", default = 1.05, range(min = 0.7, max = 1.8), advanced)]
    pub line_height: f64,
    #[setting(
        "Side margin",
        default = 8.0,
        range(min = 0, max = 30),
        unit = "%",
        advanced
    )]
    pub side_margin: f64,
    #[setting(
        "Word gap",
        default = 0.28,
        range(min = 0, max = 1),
        unit = "em",
        advanced
    )]
    pub word_gap: f64,
    #[setting("Rise", default = 0.5, range(min = 0, max = 2), unit = "em", advanced)]
    pub rise: f64,
    #[setting("Tilt", default = 4.0, range(min = -30, max = 30), unit = "°", advanced)]
    pub tilt: f64,
    #[setting("Underline emphasised words", default = true, advanced)]
    pub underline_emphasis: bool,
    #[setting(
        "Fade out",
        default = 10,
        range(min = 0, max = 60),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub fade_out_in_frames: u32,
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
