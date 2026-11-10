use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};
use crate::nodes::image::RedactStyle;

#[settings(export)]
pub struct RedactBox {
    #[setting(range(min = 0, max = 1))]
    pub x: f64,
    #[setting(range(min = 0, max = 1))]
    pub y: f64,
    #[setting(range(min = 0, max = 1))]
    pub width: f64,
    #[setting(range(min = 0, max = 1))]
    pub height: f64,
}

#[settings(export)]
pub struct Redact {
    #[setting(
        "Areas to cover",
        default = vec![RedactBox { x: 0.62, y: 0.06, width: 0.3, height: 0.08 }],
        length(max = 8),
        widget = "boxes",
        time_key = "frameMs",
        help = "Drag on the picture to add an area. The areas stay in place on screen; they don't follow the camera. To hide something on a recording while the camera moves, use Redact take on the recording instead."
    )]
    pub boxes: Vec<RedactBox>,
    #[setting(
        "Frame to draw on",
        range(min = 0, max = 3_600_000),
        step = 1,
        unit = "ms",
        hidden
    )]
    pub frame_ms: u32,
    #[setting("Hide with")]
    pub style: RedactStyle,
    #[setting(
        "Pixel size",
        default = 24,
        range(min = 6, max = 72),
        step = 2,
        unit = "px",
        show_if = { "style": ["pixelate"] }
    )]
    pub block_size: u32,
    #[setting(
        "Blur",
        default = 16.0,
        range(min = 1, max = 100),
        step = 0.5,
        unit = "px",
        show_if = { "style": ["blur"] }
    )]
    pub blur: f64,
    #[setting(
        "Colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if = { "style": ["solid"] }
    )]
    pub color: String,
    #[setting(
        "Corner radius",
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        show_if = { "style": ["solid"] },
        advanced
    )]
    pub radius: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Redact>(
        element(
            ElementKind::Wrapper,
            "redact",
            "Redact",
            "Hides parts of the screen with large pixels, a blur or a solid colour, to keep keys, emails and personal details out of the film.",
            ElementCategory::Effect,
            90,
        )
        .look(NodeLook::of("lucide:grid-3x3")),
    )
}
