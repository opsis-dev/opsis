use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Starburst {
    #[default = "#18181b"]
    #[schemars(
        description = "Ray A",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color1: String,
    #[default = "#27272a"]
    #[schemars(
        description = "Ray B",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color2: String,
    #[default = 24]
    #[schemars(description = "Rays", range(min = 6, max = 64), extend("x-step" = 1))]
    pub rays: u32,
    #[default = 0.15]
    #[schemars(
        description = "Rotation speed",
        range(min = 0, max = 2),
        extend("x-step" = 0.05, "x-unit" = "°/frame")
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
