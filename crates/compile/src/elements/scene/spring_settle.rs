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
pub struct SpringSettle {
    #[default = 1.24]
    #[schemars(description = "Enter scale", range(min = 1, max = 1.8), extend("x-step" = 0.02))]
    pub enter_scale: f64,
    #[default = 3.0]
    #[schemars(
        description = "Time between items",
        range(min = 0, max = 10),
        extend(
            "x-step" = 0.5,
            "x-unit" = "frames",
            "x-help" = "How long each item in a scene waits after the one before it."
        )
    )]
    pub enter_stagger: f64,
    #[default = 0.84]
    #[schemars(description = "Exit scale", range(min = 0.5, max = 1), extend("x-step" = 0.02))]
    pub exit_scale: f64,
    #[default = 6]
    #[schemars(
        description = "Exit time",
        range(min = 2, max = 24),
        extend("x-step" = 1, "x-unit" = "frames")
    )]
    pub exit_frames: u32,
    #[default = 5]
    #[schemars(
        description = "Fade-in time",
        range(min = 0, max = 30),
        extend("x-step" = 1, "x-unit" = "frames", "x-advanced" = true)
    )]
    pub enter_fade_frames: u32,
    #[default = 30]
    #[schemars(
        description = "Spring damping",
        range(min = 1, max = 100),
        extend(
            "x-step" = 1,
            "x-help" = "Lower values bounce more as items land.",
            "x-advanced" = true
        )
    )]
    pub spring_damping: u32,
    #[default = 320]
    #[schemars(
        description = "Spring stiffness",
        range(min = 10, max = 1000),
        extend("x-step" = 10, "x-help" = "Higher values land faster.", "x-advanced" = true)
    )]
    pub spring_stiffness: u32,
    #[default = 4]
    #[schemars(
        description = "Background fade",
        range(min = 0, max = 30),
        extend("x-step" = 1, "x-unit" = "frames", "x-advanced" = true)
    )]
    pub bg_fade_frames: u32,
    #[schemars(
        description = "Loop",
        extend("x-help" = "Start again from the first scene after the last.")
    )]
    pub r#loop: bool,
}

pub fn definition() -> ElementMeta {
    meta::<SpringSettle>(
            element(
                ElementKind::Sequencer,
                "spring-settle",
                "Spring settle",
                "Plays clips as scenes: each scene shrinks away, the stage holds empty for a beat, and the next one lands. Connect Layers to make a scene whose layers land one after another.",
                ElementCategory::Scene,
                213,
            )
                .look(NodeLook::of("tabler:carousel-vertical")),
        )
}
