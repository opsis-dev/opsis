use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LensZoom {
    #[default = 9]
    #[schemars(description = "Lens steps", range(min = 4, max = 12), extend("x-step" = 1))]
    pub lens_steps: u32,
    #[default = 7]
    #[schemars(description = "Blur samples", range(min = 2, max = 12), extend("x-step" = 1))]
    pub blur_samples: u32,
    #[default = 50]
    #[schemars(description = "Shake amount", range(min = 0, max = 300), extend("x-step" = 5))]
    pub shake_amount: u32,
    #[default = 73]
    #[schemars(description = "Shake travel (px)", range(min = 0, max = 80), extend("x-step" = 1))]
    pub shake_translate_px: u32,
    #[default = true]
    #[schemars(
        description = "Colour fringe",
        extend(
            "x-help" = "Splits the colours at the edges. Off draws the transition a third as often, for heavy scenes.",
            "x-advanced" = true
        )
    )]
    pub color_split: bool,
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
