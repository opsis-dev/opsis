use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
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
pub struct FocusPull {
    #[setting("Blur", default = 16, range(min = 4, max = 32), step = 1)]
    pub blur: u32,
}

pub fn definition() -> ElementMeta {
    meta::<FocusPull>(
            element(
                ElementKind::Presentation,
                "focus-pull",
                "Focus pull",
                "A rack-focus transition that defocuses the outgoing scene into a bright blur and resolves the next one into focus.",
                ElementCategory::Transition,
                114,
            )
                .look(NodeLook::of("lucide:focus").moves(NodeMotion::RackFocus)),
        )
}
