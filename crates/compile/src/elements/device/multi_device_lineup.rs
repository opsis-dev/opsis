use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct MultiDeviceLineup {
    #[default = 1.6]
    #[schemars(
        description = "Scale",
        range(min = 0.4, max = 2),
        extend("x-step" = 0.1, "x-help" = "Overall size of the lineup.")
    )]
    pub scale: f64,
    #[default = 9]
    #[schemars(
        description = "Stagger",
        extend("x-unit" = "frames", "x-help" = "Frames between devices.", "x-advanced" = true)
    )]
    pub stagger_in_frames: u32,
    #[default = "#15171D"]
    #[schemars(
        description = "Bezel color",
        extend(
            "x-help" = "Device body.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub bezel_color: String,
    #[default = "#0B0C11"]
    #[schemars(
        description = "Screen color",
        extend(
            "x-help" = "Screen behind your content.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub screen_color: String,
    #[default = "#7A828F"]
    #[schemars(
        description = "Label color",
        extend(
            "x-help" = "Captions under the devices.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
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

pub fn definition() -> ElementMeta {
    meta::<MultiDeviceLineup>(
        element(
            ElementKind::Wrapper,
            "multi-device-lineup",
            "Device lineup",
            "Shows the clip on a phone, a tablet and a laptop side by side.",
            ElementCategory::Device,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("tabler:devices")),
    )
}
