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
pub struct InkArrow {
    #[default = 0.35]
    #[schemars(description = "Curvature", range(min = -1, max = 1), extend("x-step" = 0.05))]
    pub curvature: f64,
    #[default = 8]
    #[schemars(description = "Brush width", range(min = 2, max = 24), extend("x-step" = 1))]
    pub stroke_width: u32,
    #[default = 0.2]
    #[schemars(
        description = "Opening pressure",
        range(min = 0.05, max = 1),
        extend("x-step" = 0.05)
    )]
    pub pressure: f64,
    #[default = 1.0]
    #[schemars(
        description = "Closing pressure",
        range(min = 0.05, max = 1),
        extend("x-step" = 0.05)
    )]
    pub release: f64,
    #[default = 1.0]
    #[schemars(description = "Grain", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub grain: f64,
    #[default = 24]
    #[schemars(description = "Head size", range(min = 8, max = 48), extend("x-step" = 1))]
    pub head_size: u32,
    #[default = 1.2]
    #[schemars(
        description = "Draw time",
        range(min = 0.4, max = 3),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub draw_dur: f64,
    #[default = 0.4]
    #[schemars(
        description = "Arrowhead time",
        range(min = 0.1333, max = 1.6),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub head_dur: f64,
    #[default = "#26242c"]
    #[schemars(
        description = "Ink color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "arrow"]
    #[schemars(description = "Seed")]
    pub seed: String,
    #[default = 0.1]
    #[schemars(
        description = "Time per pose",
        range(min = 0.0333, max = 0.2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub step: f64,
    #[default = 0.3]
    #[schemars(
        description = "From",
        range(min = 0, max = 1),
        extend(
            "x-step" = 0.01,
            "x-help" = "Click the frame to choose the spot.",
            "x-widget" = "spot"
        )
    )]
    pub from_x: f64,
    #[default = 0.72]
    #[schemars(
        description = "From vertical",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-hidden" = true)
    )]
    pub from_y: f64,
    #[default = 0.48]
    #[schemars(
        description = "To",
        range(min = 0, max = 1),
        extend(
            "x-step" = 0.01,
            "x-help" = "Click the frame to choose the spot.",
            "x-widget" = "spot"
        )
    )]
    pub to_x: f64,
    #[default = 0.5]
    #[schemars(
        description = "To vertical",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-hidden" = true)
    )]
    pub to_y: f64,
}

pub fn definition() -> ElementMeta {
    meta::<InkArrow>(
            element(
                ElementKind::Component,
                "ink-arrow",
                "Ink arrow",
                "A hand-drawn brush arrow that sweeps itself from one point to another, thickening into the head that pops on the final poses.",
                ElementCategory::Overlay,
                150,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("tabler:arrow-curve-right").moves(NodeMotion::Draw)),
        )
}
