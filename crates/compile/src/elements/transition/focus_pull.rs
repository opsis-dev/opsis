use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct FocusPull {
    #[setting("Blur", default = 16, range(min = 4, max = 32), step = 1)]
    pub blur: u32,
    #[setting(
        "Glow",
        default = 30.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub glow: f64,
    #[setting("Zoom", default = 5.0, range(min = 0, max = 20), unit = "%", advanced)]
    pub zoom: f64,
}

pub fn definition() -> ElementMeta {
    meta::<FocusPull>(
            element(
                ElementKind::Presentation,
                "focus-pull",
                "Focus pull",
                "A rack-focus transition that defocuses the outgoing scene into a bright blur and resolves the next one into focus.",
                ElementCategory::Transition,
                46,
            )
                .look(NodeLook::of("lucide:focus")),
        )
}
