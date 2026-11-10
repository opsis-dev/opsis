use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Direction;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct Flip {
    #[setting("Direction", default = Direction::FromRight)]
    pub direction: Direction,
    #[setting(
        "Perspective",
        default = 1000,
        range(min = 200, max = 5000),
        step = 1,
        unit = "px",
        advanced
    )]
    pub perspective: u32,
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
        .look(NodeLook::of(
            "lucide:triangles-centerline-dashed-horizontal",
        )),
    )
}
