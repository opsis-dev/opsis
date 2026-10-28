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
pub struct Image {
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub src: String,
    #[default(Fit::Contain)]
    #[schemars(description = "Fit")]
    pub fit: Fit,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
    #[schemars(description = "Slow push-in")]
    pub ken_burns: bool,
}

pub fn definition() -> ElementMeta {
    meta::<Image>(
        element(
            ElementKind::Component,
            "image",
            "Image",
            "Shows an image from your project.",
            ElementCategory::Media,
            90,
        )
        .look(NodeLook::of("lucide:image")),
    )
}
