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
pub struct TrackingIn {
    #[setting("Text", default = "tracking in")]
    pub text: String,
    #[setting(
        "Start tracking (em)",
        default = 0.5,
        range(min = 0, max = 2),
        step = 0.05
    )]
    pub start_tracking: f64,
    #[setting("Start blur", default = 12, range(min = 0, max = 40), step = 1)]
    pub start_blur: u32,
    #[setting("Font size", default = 96, range(min = 12, max = 200), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W700)]
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
    meta::<TrackingIn>(
        element(
            ElementKind::Component,
            "tracking-in",
            "Tracking in",
            "Letter-spacing collapses and blur clears with a springy motion.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Tracking)),
    )
}
