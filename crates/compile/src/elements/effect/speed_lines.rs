use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;
use serde_json::json;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum SpeedLinesMode {
    Trail,
    Focus,
}

#[settings(export)]
pub struct SpeedLines {
    #[setting("Mode", default = SpeedLinesMode::Trail)]
    pub mode: SpeedLinesMode,
    #[setting("Lines", default = 12, range(min = 3, max = 32), step = 1)]
    pub count: u32,
    #[setting("Stroke weight", default = 2.0, range(min = 1, max = 8), step = 0.5)]
    pub weight: f64,
    #[setting("Spread", default = 96, range(min = 16, max = 320), step = 4)]
    pub spread: u32,
    #[setting(
        "Travel frames",
        default = 14,
        range(min = 6, max = 40),
        step = 1,
        advanced
    )]
    pub duration: u32,
    #[setting(
        "Bounce",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub bounce: f64,
    #[setting("Point x", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Point y", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
    #[setting("Delay", range(min = 0, max = 60), step = 1, advanced)]
    pub delay: u32,
    #[setting("Line color", default = "#ffffff", check = colour)]
    pub color: String,
    #[setting("Seed", default = 1, range(min = 1), step = 1, advanced)]
    pub seed: u32,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Start x", default = -0.25, range(min = -0.5, max = 1.5))]
    pub from_x: f64,
    #[setting("Start y", default = 0.5, range(min = -0.5, max = 1.5))]
    pub from_y: f64,
    #[setting(
        "Hold on the point",
        default = 0.2667,
        range(min = 0.1, max = 2),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub hold: f64,
    #[setting(
        "Streak length",
        default = 1.6,
        range(min = 0.5, max = 4),
        unit = "×",
        advanced
    )]
    pub streak_length: f64,
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
