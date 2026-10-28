use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Fit;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Video {
    #[schemars(description = "Video", extend("x-widget" = "asset", "x-accept" = "video/*"))]
    pub src: String,
    #[default(Fit::Cover)]
    #[schemars(description = "Fit")]
    pub fit: Fit,
    #[schemars(description = "Loop")]
    pub r#loop: bool,
    #[default = true]
    #[schemars(description = "Muted")]
    pub muted: bool,
    #[default = 1.0]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.05))]
    pub playback_rate: f64,
    #[schemars(
        description = "Start at",
        range(min = 0, max = 3600000),
        extend("x-step" = 100, "x-unit" = "ms")
    )]
    pub trim_before_ms: u32,
}

pub fn definition() -> ElementMeta {
    meta::<Video>(
        element(
            ElementKind::Component,
            "video",
            "Video",
            "Plays a video from your project.",
            ElementCategory::Media,
            150,
        )
        .look(NodeLook::of("lucide:video")),
    )
}
