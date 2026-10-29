use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum AnimationFit {
    Contain,
    Cover,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Lottie {
    #[setting("Animation file", widget = "asset", accept = ".json,.lottie")]
    pub src: String,
    #[setting("Loop", default = true)]
    pub r#loop: bool,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Play backwards")]
    pub backwards: bool,
    #[setting(
        "Fit",
        default = AnimationFit::Contain,
        labels = { "contain": "Show all", "cover": "Fill the frame" }
    )]
    pub fit: AnimationFit,
    #[setting("Background", default = "transparent")]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Lottie>(
        element(
            ElementKind::Component,
            "lottie",
            "Animation",
            "Plays an animation file from your project, looped or once, at any speed.",
            ElementCategory::Overlay,
            90,
        )
        .look(NodeLook::of("lucide:shapes")),
    )
}
