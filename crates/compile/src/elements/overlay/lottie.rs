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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Lottie {
    #[schemars(
        description = "Animation file",
        extend("x-widget" = "asset", "x-accept" = ".json,.lottie")
    )]
    pub src: String,
    #[default = true]
    #[schemars(description = "Loop")]
    pub r#loop: bool,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
    #[schemars(description = "Play backwards")]
    pub backwards: bool,
    #[default(AnimationFit::Contain)]
    #[schemars(
        description = "Fit",
        extend("x-labels" = { "contain": "Show all", "cover": "Fill the frame" })
    )]
    pub fit: AnimationFit,
    #[default = "transparent"]
    #[schemars(description = "Background")]
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
