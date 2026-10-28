use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TransitionBlinds {
    #[default(Orientation::Horizontal)]
    #[schemars(
        description = "Orientation",
        extend("x-help" = "Horizontal slats stack top to bottom and sweep sideways.")
    )]
    pub orientation: Orientation,
    #[default = 12]
    #[schemars(
        description = "Slats",
        extend("x-help" = "Number of slats. Above ~24 it stops reading as slats at 1080p.")
    )]
    pub slats: u32,
    #[default = 0.45]
    #[schemars(
        description = "Stagger",
        extend(
            "x-step" = 0.01,
            "x-help" = "Share of the window the cascade spans. 0 opens every slat together.",
            "x-advanced" = true
        )
    )]
    pub stagger: f64,
    #[schemars(
        description = "Alternate",
        extend("x-help" = "Reverse alternate slats for a woven counter-sweep.")
    )]
    pub alternate: bool,
    #[default = 0.06]
    #[schemars(
        description = "Edge softness",
        extend(
            "x-step" = 0.01,
            "x-help" = "Soft leading edge per slat, as a share of its length.",
            "x-advanced" = true
        )
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
