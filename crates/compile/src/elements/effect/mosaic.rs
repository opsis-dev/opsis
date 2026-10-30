use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MosaicBox {
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
pub struct Mosaic {
    #[setting(
        "Areas to cover",
        default = vec![MosaicBox { x: 0.62, y: 0.06, width: 0.3, height: 0.08 }],
        length(max = 8),
        widget = "boxes",
        time_key = "frameMs",
        help = "Drag on the picture to add an area. The areas stay in place on screen; they don't follow the camera. To hide something on a recording while the camera moves, use Redact take on the recording instead."
    )]
    pub boxes: Vec<MosaicBox>,
    #[setting(
        "Frame to draw on",
        range(min = 0, max = 3600000),
        step = 1,
        unit = "ms",
        hidden
    )]
    pub frame_ms: u32,
    #[setting("Block size", default = 24, range(min = 6, max = 72), step = 2)]
    pub block_size: u32,
}

pub fn definition() -> ElementMeta {
    meta::<Mosaic>(element(ElementKind::Wrapper, "mosaic",
            "Mosaic",
            "Covers parts of the screen with coarse blocks, to hide keys, emails and personal details.",
            ElementCategory::Effect,
            90).look(NodeLook::of("lucide:grid-3x3")))
}
