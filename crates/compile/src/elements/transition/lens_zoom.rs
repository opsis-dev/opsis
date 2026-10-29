use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LensZoom {
    #[setting(
        "Lens steps",
        default = 9,
        range(min = 4, max = 12),
        step = 1,
        advanced
    )]
    pub lens_steps: u32,
    #[setting(
        "Blur samples",
        default = 7,
        range(min = 2, max = 12),
        step = 1,
        advanced
    )]
    pub blur_samples: u32,
    #[setting("Shake amount", default = 50, range(min = 0, max = 300), step = 5)]
    pub shake_amount: u32,
    #[setting(
        "Shake travel (px)",
        default = 73,
        range(min = 0, max = 80),
        step = 1,
        advanced
    )]
    pub shake_translate_px: u32,
    #[setting(
        "Colour fringe",
        default = true,
        help = "Splits the colours at the edges. Off draws the transition a third as often, for heavy scenes.",
        advanced
    )]
    pub color_split: bool,
    #[setting("Zoom", default = 150.0, range(min = 100, max = 300), unit = "%")]
    pub zoom: f64,
    #[setting("Zoom blur", default = 60.0, range(min = 0, max = 100))]
    pub zoom_blur: f64,
    #[setting(
        "Distortion",
        default = 160.0,
        range(min = 0, max = 178),
        unit = "°",
        advanced
    )]
    pub distortion: f64,
    #[setting("Twist", default = -15.0, range(min = -90, max = 90), unit = "°", advanced)]
    pub twist: f64,
    #[setting(
        "Pull back first",
        default = 100.0,
        range(min = 50, max = 100),
        unit = "%",
        advanced
    )]
    pub pull_back: f64,
    #[setting(
        "Shake speed",
        default = 11.2,
        range(min = 1, max = 30),
        unit = "per second",
        advanced
    )]
    pub shake_speed: f64,
    #[setting(
        "Shake rotation",
        default = 1.1,
        range(min = 0, max = 10),
        unit = "°",
        advanced
    )]
    pub shake_rotation: f64,
    #[setting(
        "Fringe amount",
        default = 1.0,
        range(min = 0, max = 5),
        unit = "%",
        advanced
    )]
    pub fringe: f64,
    #[setting("Letterbox", default = 0.0, range(min = 0, max = 4), advanced)]
    pub letterbox: f64,
    #[setting("Background", default = "#000000", widget = "color", check = colour, advanced)]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<LensZoom>(
            element(
                ElementKind::Presentation,
                "lens-zoom",
                "Lens zoom",
                "A hard cut disguised as a lens punch-in: barrel distortion, zoom blur, chromatic split and camera shake around the splice.",
                ElementCategory::Transition,
                112,
            )
                .look(NodeLook::of("lucide:zoom-in").moves(NodeMotion::ZoomThrough)),
        )
}
