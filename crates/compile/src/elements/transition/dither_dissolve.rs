use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DitherShape;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DitherDissolve {
    #[setting("Shape", default = DitherShape::Simplex)]
    pub shape: DitherShape,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting("Ink", default = "#8f88ae", check = colour)]
    pub color_front: String,
    #[setting(
        "Speed",
        default = 1.5,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<DitherDissolve>(
        element(
            ElementKind::Presentation,
            "dither-dissolve",
            "Dither dissolve",
            "A dithering shader transition that dissolves one scene into the next.",
            ElementCategory::Transition,
            100,
        )
        .look(NodeLook::of("lucide:grip").moves(NodeMotion::Dissolve)),
    )
}
