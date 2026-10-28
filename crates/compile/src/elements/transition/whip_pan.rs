use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum WhipDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct WhipPan {
    #[default(WhipDirection::Left)]
    #[schemars(description = "Direction")]
    pub direction: WhipDirection,
    #[default = 24]
    #[schemars(description = "Blur", range(min = 0, max = 48), extend("x-step" = 2))]
    pub blur: u32,
}

pub fn definition() -> ElementMeta {
    meta::<WhipPan>(
            element(
                ElementKind::Presentation,
                "whip-pan",
                "Whip pan",
                "A whip-pan transition that flies both scenes through the frame in one move with velocity-based motion blur.",
                ElementCategory::Transition,
                114,
            )
                .look(NodeLook::of("lucide:move-horizontal").moves(NodeMotion::Whip)),
        )
}
