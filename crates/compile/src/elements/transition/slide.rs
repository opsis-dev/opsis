use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Direction;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Slide {
    #[default(Direction::FromRight)]
    #[schemars(description = "Direction")]
    pub direction: Direction,
}

pub fn definition() -> ElementMeta {
    meta::<Slide>(
        element(
            ElementKind::Presentation,
            "slide",
            "Slide",
            "",
            ElementCategory::Transition,
            20,
        )
        .look(NodeLook::of("tabler:transition-right").moves(NodeMotion::SlideThrough)),
    )
}
