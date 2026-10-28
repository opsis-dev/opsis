use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::NoSettings;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub fn definition() -> ElementMeta {
    meta::<NoSettings>(
        element(
            ElementKind::Presentation,
            "fade",
            "Fade",
            "",
            ElementCategory::Transition,
            15,
        )
        .look(NodeLook::of("lucide:blend").moves(NodeMotion::FadeThrough)),
    )
}
