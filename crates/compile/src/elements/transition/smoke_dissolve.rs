use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SmokeDissolve {
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting("Colours", default = vec!["#8f88ae".into()], length(min = 1, max = 10))]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Smoke detail", default = 3.0, range(min = 0.5, max = 8), advanced)]
    pub detail: f64,
    #[setting("Horizontal centre", default = 0.5, range(min = 0, max = 1), advanced)]
    pub origin_x: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SmokeDissolve>(
            element(
                ElementKind::Presentation,
                "smoke-dissolve",
                "Smoke dissolve",
                "A smoke-ring shader transition that expands the ring outward to reveal the next scene.",
                ElementCategory::Transition,
                116,
            )
                .look(NodeLook::of("lucide:cloud-fog").moves(NodeMotion::Dissolve)),
        )
}
