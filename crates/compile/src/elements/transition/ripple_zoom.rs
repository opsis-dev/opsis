use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct RippleZoom {
    #[setting("Zoom", default = 4.0, range(min = 1, max = 8), step = 0.1, advanced)]
    pub zoom: f64,
    #[setting("Intensity", default = 0.5, range(min = 0, max = 1), step = 0.05)]
    pub intensity: f64,
    #[setting(
        "Softness",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()],
        length(min = 2, max = 7)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Grain", default = 0.5, range(min = 0, max = 1), step = 0.01, advanced)]
    pub noise: f64,
    #[setting("Horizontal centre", default = 0.5, range(min = 0, max = 1), advanced)]
    pub origin_x: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RippleZoom>(
            element(
                ElementKind::Presentation,
                "ripple-zoom",
                "Ripple zoom",
                "A grain-gradient ripple transition that flies the camera through the center of the rings while the next scene approaches from the depth.",
                ElementCategory::Transition,
                88,
            )
                .look(NodeLook::of("tabler:ripple").moves(NodeMotion::Ripple)),
        )
}
