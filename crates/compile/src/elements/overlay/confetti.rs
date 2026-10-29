use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Confetti {
    #[setting("Particles", default = 140, range(min = 10, max = 400), step = 10)]
    pub particle_count: u32,
    #[setting("Power", default = 17, range(min = 4, max = 40), step = 1)]
    pub power: u32,
    #[setting("Gravity", default = 0.45, range(min = 0, max = 1.5), step = 0.05)]
    pub gravity: f64,
    #[setting("Size", default = 13, range(min = 4, max = 30), step = 1)]
    pub size: u32,
    #[setting("Seed", default = 1, range(min = 1), step = 1)]
    pub seed: u32,
    #[setting(
        "Colours",
        default = vec![
            "#1d9bf0".into(),
            "#ff5da2".into(),
            "#ffd23f".into(),
            "#22c55e".into(),
            "#a855f7".into(),
            "#ff7a45".into(),
        ],
        length(min = 1, max = 12)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Burst from",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub origin_x: f64,
    #[setting(
        "Burst from vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub origin_y: f64,
    #[setting(
        "How long pieces last",
        default = 3.0,
        range(min = 0.3333, max = 20),
        step = 0.01,
        unit = "s",
        frames_at = 30
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
