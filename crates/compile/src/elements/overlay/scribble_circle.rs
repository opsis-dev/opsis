use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
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
pub struct ScribbleCircle {
    #[setting("Laps", default = 1.15, range(min = 0.5, max = 3), step = 0.05)]
    pub laps: f64,
    #[setting("Brush width", default = 14, range(min = 4, max = 32), step = 1)]
    pub stroke_width: u32,
    #[setting(
        "Opening pressure",
        default = 0.2,
        range(min = 0.05, max = 1),
        step = 0.05
    )]
    pub pressure: f64,
    #[setting("Grain", default = 1.0, range(min = 0, max = 2), step = 0.1)]
    pub grain: f64,
    #[setting("Poses to draw", default = 10, range(min = 2, max = 20), step = 1)]
    pub duration_steps: u32,
    #[setting("Ink color", default = "#6f7f35", check = colour)]
    pub color: String,
    #[setting("Seed", default = "scribble")]
    pub seed: String,
    #[setting(
        "Time per pose",
        default = 0.1,
        range(min = 0.0333, max = 0.2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub step: f64,
    #[setting(
        "Around",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub x: f64,
    #[setting(
        "Around vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub y: f64,
    #[setting(
        "Width",
        default = 420,
        range(min = 20, max = 3840),
        step = 10,
        unit = "px"
    )]
    pub width: u32,
    #[setting(
        "Height",
        default = 220,
        range(min = 20, max = 2160),
        step = 10,
        unit = "px"
    )]
    pub height: u32,
    #[setting(
        "Closing pressure",
        default = 1.0,
        range(min = 0.05, max = 1),
        step = 0.05
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
