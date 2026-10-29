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
#[serde(rename_all = "kebab-case")]
pub enum MockupDevice {
    Phone,
    Browser,
    Laptop,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceMockupZoom {
    #[setting(
        "Media",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the screen when no clip is connected. Leave empty for a built-in demo screen."
    )]
    pub src: String,
    #[setting(
        "Title",
        help = "Optional headline. When omitted, the device remains the sole focal point."
    )]
    pub title: String,
    #[setting("Subtitle", help = "Optional supporting line under the headline.")]
    pub subtitle: String,
    #[setting("Small heading", help = "Optional accent label above the headline.")]
    pub eyebrow: String,
    #[setting(
        "Device",
        default = MockupDevice::Laptop,
        help = "Mockup shell. Laptop includes a physical base, browser renders chrome only, phone renders a handheld frame."
    )]
    pub device: MockupDevice,
    #[setting("Background color", default = "#080810", check = colour)]
    pub background_color: String,
    #[setting("Accent color", default = "#e8b86d", check = colour)]
    pub accent_color: String,
    #[setting("Text color", default = "#f8fafc", check = colour)]
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
