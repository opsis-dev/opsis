use opsis_macros::{choice, settings};
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
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

#[settings(export)]
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
