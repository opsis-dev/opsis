use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ChromaticAberrationWipe {
    #[default = 12.0]
    #[schemars(description = "Intensity")]
    pub intensity: f64,
    #[default(Orientation::Horizontal)]
    #[schemars(
        description = "Axis",
        extend("x-help" = "Axis the scenes slide along and the channels separate on.")
    )]
    pub axis: Orientation,
    #[default = 1.0]
    #[schemars(
        description = "Slide distance",
        extend(
            "x-help" = "Share of the frame the scenes travel. Below 1 shows background.",
            "x-advanced" = true
        )
    )]
    pub slide: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ChromaticAberrationWipe>(
            element(
                ElementKind::Presentation,
                "chromatic-aberration-wipe",
                "Colour split slide",
                "A fast slide that pulls the colours apart mid-move and brings them back together as it lands.",
                ElementCategory::Transition,
                14,
            )
                .look(NodeLook::of("tabler:color-filter").moves(NodeMotion::SlideThrough)),
        )
}
