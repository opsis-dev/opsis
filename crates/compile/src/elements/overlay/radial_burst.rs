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
pub struct RadialBurst {
    #[default = 8]
    #[schemars(description = "Segments", range(min = 4, max = 16), extend("x-step" = 1))]
    pub segments: u32,
    #[default = 210]
    #[schemars(description = "Burst radius", range(min = 100, max = 280), extend("x-step" = 5))]
    pub radius: u32,
    #[default = 30]
    #[schemars(description = "Thickness", range(min = 8, max = 60), extend("x-step" = 1))]
    pub thickness: u32,
    #[default = 135]
    #[schemars(description = "Rotation", range(min = -360, max = 360), extend("x-step" = 15))]
    pub rotation: i32,
    #[default = 1.0]
    #[schemars(description = "Intensity", range(min = 0, max = 1.5), extend("x-step" = 0.1))]
    pub intensity: f64,
    #[default = 110]
    #[schemars(description = "Ribbon twist", range(min = -180, max = 180), extend("x-step" = 10))]
    pub twist: i32,
    #[default = 3]
    #[schemars(description = "Contour echoes", range(min = 0, max = 5), extend("x-step" = 1))]
    pub echoes: u32,
    #[default = "#ffffff"]
    #[schemars(
        description = "Shape color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#f4e4a7"]
    #[schemars(
        description = "Accent color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent_color: String,
    #[default = "#a800b7"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background_color: String,
    #[schemars(description = "Loop animation")]
    pub r#loop: bool,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RadialBurst>(
            element(
                ElementKind::Component,
                "radial-burst",
                "Radial burst",
                "Sculptural ribbons twist into a tilted ring and sweep outward through contour echoes for an expressive product introduction.",
                ElementCategory::Overlay,
                120,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:sparkle")),
        )
}
