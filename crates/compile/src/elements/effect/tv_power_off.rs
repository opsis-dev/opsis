use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct TvPowerOff {
    #[setting("Duration", default = 18, range(min = 8, max = 30), step = 1)]
    pub duration_in_frames: u32,
    #[setting("Surge", default = 1.0, range(min = 0, max = 2), step = 0.05, advanced)]
    pub gain: f64,
    #[setting("Afterglow", default = 1.0, range(min = 0, max = 2), step = 0.05)]
    pub afterglow: f64,
    #[setting("Phosphor", default = "#d8ecff", check = colour)]
    pub phosphor: String,
    #[setting("Screen off colour", default = "#000000", widget = "color", check = colour, advanced)]
    pub screen_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<TvPowerOff>(
            element(
                ElementKind::Wrapper,
                "tv-power-off",
                "TV power off",
                "End a scene the way a tube does: the picture collapses into a glowing horizontal line, the line pinches to a phosphor dot, the dot decays, and the frame is left dead black.",
                ElementCategory::Effect,
                62,
            )
                .look(NodeLook::of("lucide:tv").moves(NodeMotion::Collapse)),
        )
}
