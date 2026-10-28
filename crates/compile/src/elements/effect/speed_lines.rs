use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum SpeedLinesMode {
    Trail,
    Focus,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeedLines {
    #[default(SpeedLinesMode::Trail)]
    #[schemars(description = "Mode")]
    pub mode: SpeedLinesMode,
    #[default = 12]
    #[schemars(description = "Lines", range(min = 3, max = 32), extend("x-step" = 1))]
    pub count: u32,
    #[default = 2.0]
    #[schemars(description = "Stroke weight", range(min = 1, max = 8), extend("x-step" = 0.5))]
    pub weight: f64,
    #[default = 96]
    #[schemars(description = "Spread", range(min = 16, max = 320), extend("x-step" = 4))]
    pub spread: u32,
    #[default = 14]
    #[schemars(description = "Travel frames", range(min = 6, max = 40), extend("x-step" = 1))]
    pub duration: u32,
    #[default = 0.5]
    #[schemars(description = "Bounce", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub bounce: f64,
    #[default = 0.5]
    #[schemars(description = "Point x", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Point y", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub y: f64,
    #[schemars(description = "Delay", range(min = 0, max = 60), extend("x-step" = 1))]
    pub delay: u32,
    #[default = "#ffffff"]
    #[schemars(
        description = "Line color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = 1]
    #[schemars(description = "Seed", range(min = 1), extend("x-step" = 1))]
    pub seed: u32,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SpeedLines>(
            element(
                ElementKind::Component,
                "speed-lines",
                "Speed lines",
                "Streaks shoot across the frame toward a point and die as they land, or radial focus lines rush in from the edges and snap onto one point for an impact frame. Lay it over a clip with Layers.",
                ElementCategory::Effect,
                55,
            )
                .authored_clock()
                .fixed("children", json!(null))
                .look(NodeLook::of("lucide:chevrons-right")),
        )
}
