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
pub struct Starburst {
    #[setting("Ray A", default = "#18181b", check = colour)]
    pub color1: String,
    #[setting("Ray B", default = "#27272a", check = colour)]
    pub color2: String,
    #[setting("Rays", default = 24, range(min = 6, max = 64), step = 1)]
    pub rays: u32,
    #[setting(
        "Rotation speed",
        default = 0.15,
        range(min = 0, max = 2),
        step = 0.05,
        unit = "°/frame"
    )]
    pub rotation_speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Starburst>(
        element(
            ElementKind::Component,
            "starburst",
            "Starburst",
            "Rays spreading from the centre.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:sun")),
    )
}
