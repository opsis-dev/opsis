use opsis_macros::settings;
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
pub enum WipeDirection {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
    FromTopLeft,
    FromTopRight,
    FromBottomLeft,
    FromBottomRight,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Wipe {
    #[setting("Direction", default = WipeDirection::FromLeft)]
    pub direction: WipeDirection,
}

pub fn definition() -> ElementMeta {
    meta::<Wipe>(
        element(
            ElementKind::Presentation,
            "wipe",
            "Wipe",
            "",
            ElementCategory::Transition,
            20,
        )
        .look(NodeLook::of("lucide:square-split-horizontal").moves(NodeMotion::WipeReveal)),
    )
}
