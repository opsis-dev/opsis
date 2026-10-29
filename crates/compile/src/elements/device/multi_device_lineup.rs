use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct MultiDeviceLineup {
    #[setting(
        "Scale",
        default = 1.6,
        range(min = 0.4, max = 2),
        step = 0.1,
        help = "Overall size of the lineup."
    )]
    pub scale: f64,
    #[setting(
        "Stagger",
        default = 9,
        unit = "frames",
        help = "Frames between devices.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Bezel color", default = "#15171D", help = "Device body.", check = colour)]
    pub bezel_color: String,
    #[setting(
        "Screen color",
        default = "#0B0C11",
        help = "Screen behind your content.",
        check = colour
    )]
    pub screen_color: String,
    #[setting(
        "Label color",
        default = "#7A828F",
        help = "Captions under the devices.",
        check = colour
    )]
    pub label_color: String,
    #[setting("Phone label", default = "Phone")]
    pub phone_label: String,
    #[setting("Tablet label", default = "Tablet")]
    pub tablet_label: String,
    #[setting("Laptop label", default = "Laptop")]
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
        .fonts(&["Inter"])
        .look(NodeLook::of("tabler:devices")),
    )
}
