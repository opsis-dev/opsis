use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Direction;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Flip {
    #[setting("Direction", default = Direction::FromRight)]
    pub direction: Direction,
}

pub fn definition() -> ElementMeta {
    meta::<Flip>(
        element(
            ElementKind::Presentation,
            "flip",
            "Flip",
            "",
            ElementCategory::Transition,
            24,
        )
        .look(
            NodeLook::of("lucide:triangles-centerline-dashed-horizontal").moves(NodeMotion::Flip),
        ),
    )
}
