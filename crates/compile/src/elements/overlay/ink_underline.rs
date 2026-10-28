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
pub struct InkUnderline {
    #[default = 420]
    #[schemars(description = "Width", range(min = 100, max = 800), extend("x-step" = 10))]
    pub width: u32,
    #[default = "#6f7f35"]
    #[schemars(
        description = "Ink color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = 9]
    #[schemars(description = "Brush width", range(min = 2, max = 24), extend("x-step" = 1))]
    pub thickness: u32,
    #[default = 1.0]
    #[schemars(
        description = "Opening pressure",
        range(min = 0.05, max = 1),
        extend("x-step" = 0.05)
    )]
    pub pressure: f64,
    #[default = 0.15]
    #[schemars(
        description = "Closing pressure",
        range(min = 0.05, max = 1),
        extend("x-step" = 0.05)
    )]
    pub release: f64,
    #[default = 1.0]
    #[schemars(description = "Grain", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub grain: f64,
    #[default = 5]
    #[schemars(description = "Poses to draw", range(min = 2, max = 10), extend("x-step" = 1))]
    pub duration_steps: u32,
    #[default = "ink"]
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
        description = "Where",
        range(min = 0, max = 1),
        extend(
            "x-step" = 0.01,
            "x-help" = "Click the frame to choose the spot.",
            "x-widget" = "spot"
        )
    )]
    pub x: f64,
    #[default = 0.62]
    #[schemars(
        description = "Where vertical",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-hidden" = true)
    )]
    pub y: f64,
}

pub fn definition() -> ElementMeta {
    meta::<InkUnderline>(
            element(
                ElementKind::Component,
                "ink-underline",
                "Ink underline",
                "A hand-dragged brush underline that sweeps itself across in a few stop-motion poses, thinning to a dry tail.",
                ElementCategory::Overlay,
                120,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:underline").moves(NodeMotion::Draw)),
        )
}
