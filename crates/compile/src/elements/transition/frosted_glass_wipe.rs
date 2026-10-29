use opsis_macros::{choice, settings};
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum WipeFrom {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
}

#[settings(export)]
pub struct FrostedGlassWipe {
    #[setting("Blur", default = 20.0, help = "Frost panel blur radius.")]
    pub blur: f64,
    #[setting(
        "Panel width",
        default = 0.14,
        step = 0.01,
        help = "Sweep panel width as fraction of frame.",
        advanced
    )]
    pub panel_width: f64,
    #[setting("Direction", default = WipeFrom::FromLeft, help = "Sweep direction.")]
    pub direction: WipeFrom,
    #[setting(
        "Frost color",
        default = "rgba(255,255,255,0.12)",
        help = "Tint of the glass panel.",
        check = colour,
        advanced
    )]
    pub frost_color: String,
    #[setting("Edge colour", default = "rgba(255,255,255,0.22)", widget = "color", check = colour, advanced)]
    pub edge_color: String,
    #[setting(
        "Glow",
        default = 28,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub glow: u32,
}

pub fn definition() -> ElementMeta {
    meta::<FrostedGlassWipe>(
        element(
            ElementKind::Presentation,
            "frosted-glass-wipe",
            "Frosted glass wipe",
            "A pane of frosted glass crosses the frame and leaves the next scene behind it.",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("tabler:blur").moves(NodeMotion::WipeReveal)),
    )
}
