use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::shared::VerticalDirection;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SlotMachineRoll {
    #[setting("From", default = "$99")]
    pub from: String,
    #[setting("To", default = "$199")]
    pub to: String,
    #[setting("Font size", default = 120, range(min = 12, max = 240), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W700)]
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
        "Column stagger",
        default = 0.1333,
        range(min = 0, max = 1),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub column_stagger: f64,
    #[setting(
        "Direction",
        default = VerticalDirection::Up,
        advanced
    )]
    pub direction: VerticalDirection,
    #[setting("Letter spacing", default = -0.03, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SlotMachineRoll>(
        element(
            ElementKind::Component,
            "slot-machine-roll",
            "Slot machine roll",
            "Vertical reel scrolls characters from one value to another.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Roll)),
    )
}
