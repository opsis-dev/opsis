use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TransitionBlinds {
    #[setting(
        "Orientation",
        default = Orientation::Horizontal,
        help = "Horizontal slats stack top to bottom and sweep sideways."
    )]
    pub orientation: Orientation,
    #[setting(
        "Slats",
        default = 12,
        help = "Number of slats. Above ~24 it stops reading as slats at 1080p."
    )]
    pub slats: u32,
    #[setting(
        "Stagger",
        default = 0.45,
        step = 0.01,
        help = "Share of the window the cascade spans. 0 opens every slat together.",
        advanced
    )]
    pub stagger: f64,
    #[setting(
        "Alternate",
        help = "Reverse alternate slats for a woven counter-sweep."
    )]
    pub alternate: bool,
    #[setting(
        "Edge softness",
        default = 0.06,
        step = 0.01,
        help = "Soft leading edge per slat, as a share of its length.",
        advanced
    )]
    pub edge_softness: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TransitionBlinds>(
        element(
            ElementKind::Presentation,
            "transition-blinds",
            "Blinds",
            "Slats open one after another to reveal the next scene.",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("lucide:blinds").moves(NodeMotion::Shutter)),
    )
}
