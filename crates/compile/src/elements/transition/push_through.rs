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
pub struct PushThrough {
    #[default = 2.4]
    #[schemars(description = "Zoom", range(min = 1.5, max = 4), extend("x-step" = 0.1))]
    pub zoom: f64,
    #[default = 14]
    #[schemars(description = "Blur", range(min = 0, max = 30), extend("x-step" = 1))]
    pub blur: u32,
}

pub fn definition() -> ElementMeta {
    meta::<PushThrough>(
            element(
                ElementKind::Presentation,
                "push-through",
                "Push through",
                "A dolly transition that blows the outgoing scene past the camera while the next one approaches from the depth.",
                ElementCategory::Transition,
                112,
            )
                .look(NodeLook::of("lucide:arrow-right-to-line").moves(NodeMotion::Push)),
        )
}
