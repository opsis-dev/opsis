use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{BlendMode, Colour};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LightLeak {
    #[setting("Seed", default = 3, range(min = 0, max = 100), step = 1)]
    pub seed: u32,
    #[setting(
        "Colors",
        default = leak_colours(),
        length(min = 1, max = 4),
        help = "The light's colours, one glow each."
    )]
    pub colors: Vec<Colour>,
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

pub fn leak_colours() -> Vec<Colour> {
    vec!["#ff8a3d".into(), "#ffd166".into(), "#ff4f7b".into()]
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
