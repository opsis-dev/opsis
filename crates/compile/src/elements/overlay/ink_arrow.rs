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
pub struct InkArrow {
    #[setting("Curvature", default = 0.35, range(min = -1, max = 1), step = 0.05)]
    pub curvature: f64,
    #[setting("Brush width", default = 8, range(min = 2, max = 24), step = 1)]
    pub stroke_width: u32,
    #[setting(
        "Opening pressure",
        default = 0.2,
        range(min = 0.05, max = 1),
        step = 0.05
    )]
    pub pressure: f64,
    #[setting(
        "Closing pressure",
        default = 1.0,
        range(min = 0.05, max = 1),
        step = 0.05
    )]
    pub release: f64,
    #[setting("Grain", default = 1.0, range(min = 0, max = 2), step = 0.1)]
    pub grain: f64,
    #[setting("Head size", default = 24, range(min = 8, max = 48), step = 1)]
    pub head_size: u32,
    #[setting(
        "Draw time",
        default = 1.2,
        range(min = 0.4, max = 3),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub draw_dur: f64,
    #[setting(
        "Arrowhead time",
        default = 0.4,
        range(min = 0.1333, max = 1.6),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub head_dur: f64,
    #[setting("Ink color", default = "#26242c", check = colour)]
    pub color: String,
    #[setting("Seed", default = "arrow")]
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
        "From",
        default = 0.3,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub from_x: f64,
    #[setting(
        "From vertical",
        default = 0.72,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub from_y: f64,
    #[setting(
        "To",
        default = 0.48,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub to_x: f64,
    #[setting(
        "To vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
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
