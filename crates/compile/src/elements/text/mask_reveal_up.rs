use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::{FontWeight, TextAlign};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MaskRevealUp {
    #[setting(
        "Text",
        default = "Designed to move.\nBuilt to focus.",
        widget = "textarea"
    )]
    pub text: String,
    #[setting(
        "Distance",
        default = 30,
        range(min = 0, max = 120),
        step = 1,
        advanced
    )]
    pub distance: u32,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Align",
        default = TextAlign::Center
    )]
    pub align: TextAlign,
    #[setting(
        "Line stagger",
        default = 0.1,
        range(min = 0, max = 1),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub line_stagger: f64,
    #[setting(
        "Exit distance",
        default = 22,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub exit_distance: u32,
    #[setting(
        "Blur",
        default = 6,
        range(min = 0, max = 30),
        step = 1,
        unit = "px",
        advanced
    )]
    pub blur: u32,
    #[setting("Line height", default = 1.1, range(min = 0.7, max = 2), advanced)]
    pub line_height: f64,
}

pub fn definition() -> ElementMeta {
    meta::<MaskRevealUp>(
        element(
            ElementKind::Component,
            "mask-reveal-up",
            "Mask reveal up",
            "Lines reveal upward with a soft masked feel and compact stagger.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::MaskUp)),
    )
}
