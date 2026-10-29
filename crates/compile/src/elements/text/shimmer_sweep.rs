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
pub struct ShimmerSweep {
    #[setting("Text", default = "Generating")]
    pub text: String,
    #[setting("Base color", default = "#3f3f46", check = colour)]
    pub base_color: String,
    #[setting("Shine color", default = "#fafafa", check = colour)]
    pub shine_color: String,
    #[setting("Font size", default = 96, range(min = 12, max = 200), step = 1)]
    pub font_size: u32,
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
    meta::<ShimmerSweep>(
        element(
            ElementKind::Component,
            "shimmer-sweep",
            "Shimmer sweep",
            "A band of light sweeps across the text.",
            ElementCategory::Text,
            120,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Shimmer)),
    )
}
