use opsis_macros::settings;
use opsis_types::looks::NodeLook;
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
pub struct SpringSettle {
    #[setting("Enter scale", default = 1.24, range(min = 1, max = 1.8), step = 0.02)]
    pub enter_scale: f64,
    #[setting(
        "Time between items",
        default = 3.0,
        range(min = 0, max = 10),
        step = 0.5,
        unit = "frames",
        help = "How long each item in a scene waits after the one before it."
    )]
    pub enter_stagger: f64,
    #[setting("Exit scale", default = 0.84, range(min = 0.5, max = 1), step = 0.02)]
    pub exit_scale: f64,
    #[setting(
        "Exit time",
        default = 6,
        range(min = 2, max = 24),
        step = 1,
        unit = "frames"
    )]
    pub exit_frames: u32,
    #[setting(
        "Fade-in time",
        default = 5,
        range(min = 0, max = 30),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub enter_fade_frames: u32,
    #[setting(
        "Spring damping",
        default = 30,
        range(min = 1, max = 100),
        step = 1,
        help = "Lower values bounce more as items land.",
        advanced
    )]
    pub spring_damping: u32,
    #[setting(
        "Spring stiffness",
        default = 320,
        range(min = 10, max = 1000),
        step = 10,
        help = "Higher values land faster.",
        advanced
    )]
    pub spring_stiffness: u32,
    #[setting(
        "Background fade",
        default = 4,
        range(min = 0, max = 30),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub bg_fade_frames: u32,
    #[setting("Loop", help = "Start again from the first scene after the last.")]
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
