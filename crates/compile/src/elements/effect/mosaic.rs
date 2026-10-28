use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct MosaicBox {
    #[schemars(range(min = 0, max = 1))]
    pub x: f64,
    #[schemars(range(min = 0, max = 1))]
    pub y: f64,
    #[schemars(range(min = 0, max = 1))]
    pub width: f64,
    #[schemars(range(min = 0, max = 1))]
    pub height: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Mosaic {
    #[default(vec![MosaicBox { x: 0.62, y: 0.06, width: 0.3, height: 0.08 }])]
    #[schemars(
        description = "Areas to cover",
        length(max = 8),
        extend(
            "x-widget" = "boxes",
            "x-time-key" = "frameMs",
            "x-help" = "Drag on the picture to add an area. The areas stay in place on screen; they don't follow the camera. To hide something on a recording while the camera moves, use Blur areas on the recording instead."
        )
    )]
    pub boxes: Vec<MosaicBox>,
    #[schemars(
        description = "Frame to draw on",
        range(min = 0, max = 3600000),
        extend("x-step" = 1, "x-unit" = "ms", "x-hidden" = true)
    )]
    pub frame_ms: u32,
    #[default = 24]
    #[schemars(description = "Block size", range(min = 6, max = 72), extend("x-step" = 2))]
    pub block_size: u32,
}

pub fn definition() -> ElementMeta {
    meta::<Mosaic>(element(ElementKind::Wrapper, "mosaic",
            "Mosaic",
            "Covers parts of the screen with coarse blocks, to hide keys, emails and personal details.",
            ElementCategory::Effect,
            90).look(NodeLook::of("lucide:grid-3x3")))
}
