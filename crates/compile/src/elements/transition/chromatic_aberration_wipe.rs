use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ChromaticAberrationWipe {
    #[setting("Intensity", default = 12.0)]
    pub intensity: f64,
    #[setting(
        "Axis",
        default = Orientation::Horizontal,
        help = "Axis the scenes slide along and the channels separate on."
    )]
    pub axis: Orientation,
    #[setting(
        "Slide distance",
        default = 1.0,
        help = "Share of the frame the scenes travel. Below 1 shows background.",
        advanced
    )]
    pub slide: f64,
    #[setting("Fade while sliding", advanced)]
    pub fade: bool,
}

pub fn definition() -> ElementMeta {
    meta::<ChromaticAberrationWipe>(
            element(
                ElementKind::Presentation,
                "chromatic-aberration-wipe",
                "Colour split slide",
                "A fast slide that pulls the colours apart mid-move and brings them back together as it lands.",
                ElementCategory::Transition,
                14,
            )
                .look(NodeLook::of("tabler:color-filter").moves(NodeMotion::SlideThrough)),
        )
}
