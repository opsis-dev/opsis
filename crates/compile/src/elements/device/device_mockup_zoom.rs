use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MockupDevice {
    #[label("Phone")]
    Phone,
    #[label("Browser")]
    Browser,
    #[label("Laptop")]
    Laptop,
}

#[settings(export)]
pub struct DeviceMockupZoom {
    #[setting(
        "Device",
        default = MockupDevice::Laptop,
        help = "A laptop has a base, a browser is only its window, and a phone is held upright."
    )]
    pub device: MockupDevice,
    #[setting("Background color", default = "#080810", check = colour)]
    pub background_color: String,
    #[setting("Accent color", default = "#e8b86d", check = colour)]
    pub accent_color: String,
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
