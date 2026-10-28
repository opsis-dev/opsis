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
pub struct PageTurn {
    #[default(-7)]
    #[schemars(description = "Angle", range(min = -20, max = 20), extend("x-step" = 1))]
    pub angle: i32,
    #[default = 8]
    #[schemars(description = "Poses", range(min = 2, max = 16), extend("x-step" = 1))]
    pub poses: u32,
}

pub fn definition() -> ElementMeta {
    meta::<PageTurn>(
            element(
                ElementKind::Presentation,
                "page-turn",
                "Page turn",
                "The exiting scene swings up and away like a notebook page in stop-motion poses, staying above the scene it reveals.",
                ElementCategory::Transition,
                128,
            )
                .look(NodeLook::of("lucide:book-open").moves(NodeMotion::Flip)),
        )
}
