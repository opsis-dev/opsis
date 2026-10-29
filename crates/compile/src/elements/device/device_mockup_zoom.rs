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
    #[setting("Device color", default = "#171b26", check = colour, advanced)]
    pub device_color: String,
    #[setting("Device edge color", default = "#242a3a", check = colour, advanced)]
    pub device_edge_color: String,
    #[setting("Laptop bezel color", default = "#0d111b", check = colour, advanced)]
    pub bezel_color: String,
    #[setting("Button color", default = "#0c0e15", check = colour, advanced)]
    pub button_color: String,
    #[setting("Camera color", default = "#05060a", check = colour, advanced)]
    pub camera_color: String,
    #[setting("Phone island color", default = "#000000", check = colour, advanced)]
    pub island_color: String,
    #[setting("Lens color", default = "#1e2230", check = colour, advanced)]
    pub lens_color: String,
    #[setting(
        "Laptop base",
        default = "linear-gradient(180deg, #30384b 0%, #171b26 60%, #0b0d14 100%)",
        widget = "gradient",
        advanced
    )]
    pub base_gradient: String,
    #[setting("Second accent", default = "#2dd4bf", check = colour, advanced)]
    pub second_accent: String,
    #[setting("Third accent", default = "#f472b6", check = colour, advanced)]
    pub third_accent: String,
    #[setting("Built-in screen color", default = "#f6f1e8", check = colour, advanced)]
    pub screen_color: String,
    #[setting("Built-in screen text", default = "#111827", check = colour, advanced)]
    pub screen_text_color: String,
    #[setting("Built-in screen soft text", default = "#6b7280", check = colour, advanced)]
    pub screen_muted_color: String,
    #[setting("Background tint", default = "#5b6b9a", check = colour, advanced)]
    pub background_tint: String,
    #[setting(
        "Shadow",
        default = 0.42,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub shadow: f64,
    #[setting(
        "Accent glow",
        default = 0.2,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub glow: f64,
    #[setting("Grid", default = true, advanced)]
    pub show_grid: bool,
    #[setting(
        "Starting zoom",
        default = 1.18,
        range(min = 1, max = 2),
        step = 0.01,
        help = "How close the camera starts before it pulls back.",
        advanced
    )]
    pub pull_back_from: f64,
    #[setting(
        "Device size",
        default = 1.0,
        range(min = 0.5, max = 1.5),
        step = 0.05,
        advanced
    )]
    pub scale: f64,
    #[setting(
        "Title size",
        default = 64,
        range(min = 24, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub title_size: u32,
    #[setting(
        "Title weight",
        default = 800,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub title_weight: u32,
    #[setting(
        "Subtitle size",
        default = 24,
        range(min = 12, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub subtitle_size: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
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
                .look(NodeLook::of("lucide:laptop-minimal")),
        )
}
