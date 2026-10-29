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
pub struct Image {
    #[setting("Image", widget = "asset", accept = "image/*")]
    pub src: String,
    #[setting("Fit", default = Fit::Contain)]
    pub fit: Fit,
    #[setting("Background", default = "transparent")]
    pub background: String,
    #[setting("Slow push-in")]
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
