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
