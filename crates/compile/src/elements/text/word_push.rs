use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
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
pub struct WordPush {
    #[setting("Text", default = "move at your own pace")]
    pub text: String,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Frames / word", default = 10, range(min = 3, max = 30), step = 1)]
    pub word_gap: u32,
    #[setting(
        "Gap shrink / word",
        default = 0.8,
        range(min = 0.5, max = 1),
        step = 0.05,
        advanced
    )]
    pub accel: f64,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W400)]
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
        "Push time",
        default = 0.4,
        range(min = 0.1, max = 1.5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub push_time: f64,
    #[setting(
        "Drive-in distance",
        default = 0.5,
        range(min = 0, max = 3),
        unit = "em",
        advanced
    )]
    pub drive_distance: f64,
    #[setting("Letter spacing", default = -0.02, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<WordPush>(
            element(
                ElementKind::Component,
                "word-push",
                "Word push",
                "A phrase builds from its first word centered: each next word drives in from the right and pushes the line left, easing into the new center.",
                ElementCategory::Text,
                90,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Push)),
        )
}
