use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct PushThrough {
    #[setting("Zoom", default = 2.4, range(min = 1.5, max = 4), step = 0.1)]
    pub zoom: f64,
    #[setting("Blur", default = 14, range(min = 0, max = 30), step = 1, unit = "px")]
    pub blur: u32,
    #[setting(
        "Approach from",
        default = 0.68,
        range(min = 0.3, max = 1),
        unit = "×",
        advanced
    )]
    pub approach_from: f64,
}

pub fn definition() -> ElementMeta {
    meta::<PushThrough>(
            element(
                ElementKind::Presentation,
                "push-through",
                "Push through",
                "A dolly transition that blows the outgoing scene past the camera while the next one approaches from the depth.",
                ElementCategory::Transition,
                112,
            )
                .look(NodeLook::of("lucide:arrow-right-to-line").moves(NodeMotion::Push)),
        )
}
