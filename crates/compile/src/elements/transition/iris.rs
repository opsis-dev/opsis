use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::NoSettings;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub fn definition() -> ElementMeta {
    meta::<NoSettings>(
        element(
            ElementKind::Presentation,
            "iris",
            "Iris",
            "",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("lucide:aperture").moves(NodeMotion::Iris)),
    )
}
