use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::BlendMode;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LightLeak {
    #[setting("Seed", default = 3, range(min = 0, max = 100), step = 1)]
    pub seed: u32,
    #[setting("Hue shift", range(min = 0, max = 360), step = 1)]
    pub hue_shift: u32,
    #[setting(
        "Strength",
        default = 100.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub strength: f64,
    #[setting(
        "Blend",
        default = BlendMode::Normal,
        advanced
    )]
    pub blend: BlendMode,
}

pub fn definition() -> ElementMeta {
    meta::<LightLeak>(
        element(
            ElementKind::Component,
            "light-leak",
            "Light leak",
            "A warm light flare over the video.",
            ElementCategory::Overlay,
            45,
        )
        .look(NodeLook::of("lucide:sun-dim")),
    )
}
