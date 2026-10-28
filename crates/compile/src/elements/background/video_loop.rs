use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct VideoLoop {
    #[schemars(description = "Video", extend("x-widget" = "asset", "x-accept" = "video/*"))]
    pub src: String,
    #[default = 0.3]
    #[schemars(description = "Dim", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub dim: f64,
}

pub fn definition() -> ElementMeta {
    meta::<VideoLoop>(
        element(
            ElementKind::Component,
            "video-loop",
            "Looping video",
            "A video that loops for as long as needed.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:infinity")),
    )
}
