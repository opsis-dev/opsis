use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ZoomBlur {
    #[setting("Blur", default = 16, range(min = 0, max = 40), step = 2, unit = "px")]
    pub blur: u32,
    #[setting("Rise", range(min = 0, max = 160), step = 10, unit = "px")]
    pub rise: u32,
    #[setting("Zoom", default = 1.12, range(min = 1, max = 2), unit = "×", advanced)]
    pub zoom: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ZoomBlur>(
            element(
                ElementKind::Presentation,
                "zoom-blur",
                "Zoom blur",
                "A depth punch-in transition that pushes the outgoing scene through the viewer while the incoming one resolves out of blur.",
                ElementCategory::Transition,
                114,
            )
                .look(NodeLook::of("lucide:expand").moves(NodeMotion::ZoomBlur)),
        )
}
