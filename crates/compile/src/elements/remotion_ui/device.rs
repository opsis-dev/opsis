use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::MockupDevice;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceMockupZoom {
    #[schemars(description = "Media", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the screen when no clip is connected. Leave empty for a built-in demo screen."))]
    pub src: String,
    #[schemars(description = "Title", extend("x-help" = "Optional headline. When omitted, the device remains the sole focal point."))]
    pub title: String,
    #[schemars(description = "Subtitle", extend("x-help" = "Optional supporting line under the headline."))]
    pub subtitle: String,
    #[schemars(description = "Small heading", extend("x-help" = "Optional accent label above the headline."))]
    pub eyebrow: String,
    #[default(MockupDevice::Laptop)]
    #[schemars(description = "Device", extend("x-help" = "Mockup shell. Laptop includes a physical base, browser renders chrome only, phone renders a handheld frame."))]
    pub device: MockupDevice,
    #[default = "#080810"]
    #[schemars(description = "Background color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background_color: String,
    #[default = "#e8b86d"]
    #[schemars(description = "Accent color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = "#f8fafc"]
    #[schemars(description = "Text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub text_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MultiDeviceLineup {
    #[default = 1.6]
    #[schemars(description = "Scale", range(min = 0.4, max = 2), extend("x-step" = 0.1, "x-help" = "Overall size of the lineup."))]
    pub scale: f64,
    #[default = 9.0]
    #[schemars(description = "Stagger", extend("x-unit" = "frames", "x-help" = "Frames between devices.", "x-advanced" = true))]
    pub stagger_in_frames: f64,
    #[default = "#15171D"]
    #[schemars(description = "Bezel color", extend("x-help" = "Device body.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub bezel_color: String,
    #[default = "#0B0C11"]
    #[schemars(description = "Screen color", extend("x-help" = "Screen behind your content.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub screen_color: String,
    #[default = "#7A828F"]
    #[schemars(description = "Label color", extend("x-help" = "Captions under the devices.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub label_color: String,
    #[default = "Phone"]
    #[schemars(description = "Phone label")]
    pub phone_label: String,
    #[default = "Tablet"]
    #[schemars(description = "Tablet label")]
    pub tablet_label: String,
    #[default = "Laptop"]
    #[schemars(description = "Laptop label")]
    pub laptop_label: String,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<DeviceMockupZoom>(
            element(
                ElementKind::Wrapper,
                "device-mockup-zoom",
                "Device mockup",
                "Shows the clip on a laptop, phone or browser screen while the camera slowly pulls back.",
                C::Device,
                90,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/device-mockup-zoom", "MIT")
                .optional_clip()
                .look(NodeLook::of("lucide:laptop-minimal")),
        ),
        meta::<MultiDeviceLineup>(
            element(
                ElementKind::Wrapper,
                "multi-device-lineup",
                "Device lineup",
                "Shows the clip on a phone, a tablet and a laptop side by side.",
                C::Device,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/multi-device-lineup", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("tabler:devices")),
        ),
    ]
}
