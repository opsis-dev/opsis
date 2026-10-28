use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Confetti {
    #[default = 140]
    #[schemars(description = "Particles", range(min = 10, max = 400), extend("x-step" = 10))]
    pub particle_count: u32,
    #[default = 17]
    #[schemars(description = "Power", range(min = 4, max = 40), extend("x-step" = 1))]
    pub power: u32,
    #[default = 0.45]
    #[schemars(description = "Gravity", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub gravity: f64,
    #[default = 13]
    #[schemars(description = "Size", range(min = 4, max = 30), extend("x-step" = 1))]
    pub size: u32,
    #[default = 1]
    #[schemars(description = "Seed", range(min = 1), extend("x-step" = 1))]
    pub seed: u32,
    #[default(vec![
        "#1d9bf0".into(),
        "#ff5da2".into(),
        "#ffd23f".into(),
        "#22c55e".into(),
        "#a855f7".into(),
        "#ff7a45".into(),
    ])]
    #[schemars(description = "Colours", length(min = 1, max = 12))]
    pub colors: Vec<Colour>,
    #[default = 0.5]
    #[schemars(
        description = "Burst from",
        range(min = 0, max = 1),
        extend(
            "x-step" = 0.01,
            "x-help" = "Click the frame to choose the spot.",
            "x-widget" = "spot"
        )
    )]
    pub origin_x: f64,
    #[default = 0.5]
    #[schemars(
        description = "Burst from vertical",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-hidden" = true)
    )]
    pub origin_y: f64,
    #[default = 3.0]
    #[schemars(
        description = "How long pieces last",
        range(min = 0.3333, max = 20),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub lifetime: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Confetti>(
        element(
            ElementKind::Component,
            "confetti",
            "Confetti",
            "A burst of confetti to celebrate a moment.",
            ElementCategory::Overlay,
            90,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:party-popper")),
    )
}
