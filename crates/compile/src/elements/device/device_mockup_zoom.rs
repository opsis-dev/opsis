use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum MockupDevice {
    Phone,
    Browser,
    Laptop,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceMockupZoom {
    #[schemars(
        description = "Media",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the screen when no clip is connected. Leave empty for a built-in demo screen."
        )
    )]
    pub src: String,
    #[schemars(
        description = "Title",
        extend(
            "x-help" = "Optional headline. When omitted, the device remains the sole focal point."
        )
    )]
    pub title: String,
    #[schemars(
        description = "Subtitle",
        extend("x-help" = "Optional supporting line under the headline.")
    )]
    pub subtitle: String,
    #[schemars(
        description = "Small heading",
        extend("x-help" = "Optional accent label above the headline.")
    )]
    pub eyebrow: String,
    #[default(MockupDevice::Laptop)]
    #[schemars(
        description = "Device",
        extend(
            "x-help" = "Mockup shell. Laptop includes a physical base, browser renders chrome only, phone renders a handheld frame."
        )
    )]
    pub device: MockupDevice,
    #[default = "#080810"]
    #[schemars(
        description = "Background color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background_color: String,
    #[default = "#e8b86d"]
    #[schemars(
        description = "Accent color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent_color: String,
    #[default = "#f8fafc"]
    #[schemars(
        description = "Text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub text_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<DeviceMockupZoom>(
            element(
                ElementKind::Wrapper,
                "device-mockup-zoom",
                "Device mockup",
                "Shows the clip on a laptop, phone or browser screen while the camera slowly pulls back.",
                ElementCategory::Device,
                90,
            )
                .optional_clip()
                .fonts(&["Inter"])
                .look(NodeLook::of("lucide:laptop-minimal")),
        )
}
