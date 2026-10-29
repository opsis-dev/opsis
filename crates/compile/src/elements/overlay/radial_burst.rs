use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct RadialBurst {
    #[setting("Segments", default = 8, range(min = 4, max = 16), step = 1)]
    pub segments: u32,
    #[setting("Burst radius", default = 210, range(min = 100, max = 280), step = 5)]
    pub radius: u32,
    #[setting("Thickness", default = 30, range(min = 8, max = 60), step = 1)]
    pub thickness: u32,
    #[setting("Rotation", default = 135, range(min = -360, max = 360), step = 15, advanced)]
    pub rotation: i32,
    #[setting("Intensity", default = 1.0, range(min = 0, max = 1.5), step = 0.1)]
    pub intensity: f64,
    #[setting("Ribbon twist", default = 110, range(min = -180, max = 180), step = 10, advanced)]
    pub twist: i32,
    #[setting(
        "Contour echoes",
        default = 3,
        range(min = 0, max = 5),
        step = 1,
        advanced
    )]
    pub echoes: u32,
    #[setting("Shape color", default = "#ffffff", check = colour)]
    pub color: String,
    #[setting("Accent color", default = "#f4e4a7", check = colour)]
    pub accent_color: String,
    #[setting("Background", default = "#a800b7", check = colour)]
    pub background_color: String,
    #[setting("Loop animation")]
    pub r#loop: bool,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Echo strength", default = 0.36, range(min = 0, max = 1), advanced)]
    pub echo_opacity: f64,
    #[setting("Centre across", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Centre down", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RadialBurst>(
            element(
                ElementKind::Component,
                "radial-burst",
                "Radial burst",
                "Sculptural ribbons twist into a tilted ring and sweep outward through contour echoes for an expressive product introduction.",
                ElementCategory::Overlay,
                120,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:sparkle")),
        )
}
