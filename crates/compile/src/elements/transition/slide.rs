use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::Direction;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct Slide {
    #[setting("Direction", default = Direction::FromRight)]
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
