use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Fit;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct VideoLoop {
    #[setting("Video", widget = "asset", accept = "video/*")]
    pub src: String,
    #[setting("Dim", default = 0.3, range(min = 0, max = 1), step = 0.05)]
    pub dim: f64,
    #[setting("Dim colour", default = "#000000", widget = "color", check = colour, advanced)]
    pub dim_color: String,
    #[setting("Fit", default = Fit::Cover, advanced)]
    pub fit: Fit,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        unit = "x",
        advanced
    )]
    pub playback_rate: f64,
    #[setting(
        "Start at",
        default = 0,
        range(min = 0),
        step = 1,
        unit = "ms",
        advanced
    )]
    pub trim_before_ms: u32,
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
