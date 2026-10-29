use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
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
        check = colour,
        advanced
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
    #[setting(
        "Laptop base",
        default = "linear-gradient(180deg, #23262E 0%, #14161B 100%)",
        widget = "gradient",
        advanced
    )]
    pub base_gradient: String,
    #[setting("Phone island color", default = "#05060A", check = colour, advanced)]
    pub island_color: String,
    #[setting("Tablet camera color", default = "rgba(255,255,255,0.28)", check = colour, advanced)]
    pub camera_color: String,
    #[setting("Edge color", default = "rgba(255,255,255,0.10)", check = colour, advanced)]
    pub edge_color: String,
    #[setting(
        "Shadow",
        default = 0.55,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub shadow: f64,
    #[setting(
        "Gap",
        default = 26,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub gap: u32,
    #[setting(
        "Label size",
        default = 12,
        range(min = 8, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub label_size: u32,
    #[setting("Uppercase labels", default = true, advanced)]
    pub label_uppercase: bool,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
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
