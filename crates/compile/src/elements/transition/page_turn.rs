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
pub struct PageTurn {
    #[setting("Angle", default = -7, range(min = -20, max = 20), step = 1, unit = "°")]
    pub angle: i32,
    #[setting("Poses", default = 8, range(min = 2, max = 16), step = 1)]
    pub poses: u32,
    #[setting("Hinge position", default = 0.18, range(min = 0, max = 1), advanced)]
    pub hinge_x: f64,
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
