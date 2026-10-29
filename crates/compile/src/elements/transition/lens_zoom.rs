use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LensZoom {
    #[setting("Lens steps", default = 9, range(min = 4, max = 12), step = 1)]
    pub lens_steps: u32,
    #[setting("Blur samples", default = 7, range(min = 2, max = 12), step = 1)]
    pub blur_samples: u32,
    #[setting("Shake amount", default = 50, range(min = 0, max = 300), step = 5)]
    pub shake_amount: u32,
    #[setting("Shake travel (px)", default = 73, range(min = 0, max = 80), step = 1)]
    pub shake_translate_px: u32,
    #[setting(
        "Colour fringe",
        default = true,
        help = "Splits the colours at the edges. Off draws the transition a third as often, for heavy scenes.",
        advanced
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
