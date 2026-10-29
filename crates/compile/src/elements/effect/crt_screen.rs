use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct CrtScreen {
    #[setting("Curvature", default = 1.0, range(min = 0, max = 2), step = 0.05)]
    pub curvature: f64,
    #[setting("Scanlines", default = 1.0, range(min = 0, max = 1.5), step = 0.05)]
    pub scanlines: f64,
    #[setting("Mask pitch", default = 2, range(min = 1, max = 6), step = 1, advanced)]
    pub mask_scale: u32,
    #[setting("Vignette", default = 1.0, range(min = 0, max = 1.5), step = 0.05)]
    pub vignette: f64,
    #[setting(
        "Scanline count",
        default = 240,
        range(min = 120, max = 720),
        step = 1,
        advanced
    )]
    pub scanline_count: u32,
    #[setting("Phosphor mask", default = 0.66, range(min = 0, max = 1), advanced)]
    pub mask: f64,
    #[setting("Glow", default = 0.75, range(min = 0, max = 2), advanced)]
    pub glow: f64,
    #[setting("Flicker", default = 0.012, range(min = 0, max = 0.1), advanced)]
    pub flicker: f64,
    #[setting("Around the screen", default = "#000000", widget = "color", check = colour, advanced)]
    pub bezel_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<CrtScreen>(
            element(
                ElementKind::Wrapper,
                "crt-screen",
                "CRT screen",
                "Put a scene behind curved glass: the picture bulges out to the corners, an RGB phosphor mask and scanlines ride the curve, bright areas flood the mask, and the tube falls away into darkness at the rim.",
                ElementCategory::Effect,
                90,
            )
                .look(NodeLook::of("lucide:tv-minimal")),
        )
}
