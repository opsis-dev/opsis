use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::NoSettings;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub fn definition() -> ElementMeta {
    meta::<NoSettings>(
        element(
            ElementKind::Presentation,
            "clock-wipe",
            "Clock wipe",
            "",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("lucide:clock-fading")),
    )
}
