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
pub struct Video {
    #[setting("Video", widget = "asset", accept = "video/*")]
    pub src: String,
    #[setting("Fit", default = Fit::Cover)]
    pub fit: Fit,
    #[setting("Loop")]
    pub r#loop: bool,
    #[setting("Muted", default = true)]
    pub muted: bool,
    #[setting("Volume", default = 1.0, range(min = 0, max = 2), step = 0.05)]
    pub volume: f64,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.05)]
    pub playback_rate: f64,
    #[setting("Start at", range(min = 0, max = 3600000), step = 100, unit = "ms")]
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
