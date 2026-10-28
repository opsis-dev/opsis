use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ScribbleCircle {
    #[default = 1.15]
    #[schemars(description = "Laps", range(min = 0.5, max = 3), extend("x-step" = 0.05))]
    pub laps: f64,
    #[default = 14]
    #[schemars(description = "Brush width", range(min = 4, max = 32), extend("x-step" = 1))]
    pub stroke_width: u32,
    #[default = 0.2]
    #[schemars(
        description = "Opening pressure",
        range(min = 0.05, max = 1),
        extend("x-step" = 0.05)
    )]
    pub pressure: f64,
    #[default = 1.0]
    #[schemars(description = "Grain", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub grain: f64,
    #[default = 10]
    #[schemars(description = "Poses to draw", range(min = 2, max = 20), extend("x-step" = 1))]
    pub duration_steps: u32,
    #[default = "#6f7f35"]
    #[schemars(
        description = "Ink color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "scribble"]
    #[schemars(description = "Seed")]
    pub seed: String,
    #[default = 0.1]
    #[schemars(
        description = "Time per pose",
        range(min = 0.0333, max = 0.2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub step: f64,
    #[default = 0.5]
    #[schemars(
        description = "Around",
        range(min = 0, max = 1),
        extend(
            "x-step" = 0.01,
            "x-help" = "Click the frame to choose the spot.",
            "x-widget" = "spot"
        )
    )]
    pub x: f64,
    #[default = 0.5]
    #[schemars(
        description = "Around vertical",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-hidden" = true)
    )]
    pub y: f64,
    #[default = 420]
    #[schemars(
        description = "Width",
        range(min = 20, max = 3840),
        extend("x-step" = 10, "x-unit" = "px")
    )]
    pub width: u32,
    #[default = 220]
    #[schemars(
        description = "Height",
        range(min = 20, max = 2160),
        extend("x-step" = 10, "x-unit" = "px")
    )]
    pub height: u32,
    #[default = 1.0]
    #[schemars(
        description = "Closing pressure",
        range(min = 0.05, max = 1),
        extend("x-step" = 0.05)
    )]
    pub release: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ScribbleCircle>(
            element(
                ElementKind::Component,
                "scribble-circle",
                "Scribble circle",
                "A brush circle swept around a region of the frame, opening thin and closing thick, with grain breaking up the dry opening.",
                ElementCategory::Overlay,
                150,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:lasso").moves(NodeMotion::Draw)),
        )
}
