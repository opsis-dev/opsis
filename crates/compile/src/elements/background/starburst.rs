use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Starburst {
    #[setting("Ray A", default = "#18181b", check = colour)]
    pub color1: String,
    #[setting("Ray B", default = "#27272a", check = colour)]
    pub color2: String,
    #[setting("Rays", default = 24, range(min = 6, max = 64), step = 1)]
    pub rays: u32,
    #[setting(
        "Rotation speed",
        default = 0.15,
        range(min = 0, max = 2),
        step = 0.05,
        unit = "°/frame"
    )]
    pub rotation_speed: f64,
    #[setting(
        "Edge fade",
        default = 75.0,
        range(min = 20, max = 150),
        unit = "%",
        advanced
    )]
    pub edge_fade: f64,
    #[setting(
        "Ray width",
        default = 50.0,
        range(min = 10, max = 90),
        unit = "%",
        advanced
    )]
    pub ray_width: f64,
    #[setting(
        "Centre across",
        default = 50.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub center_x: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Starburst>(
        element(
            ElementKind::Component,
            "starburst",
            "Starburst",
            "Rays spreading from the centre.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:sun")),
    )
}
