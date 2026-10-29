use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum LaptopEntrance {
    Rise,
    Open,
    None,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum LaptopFinale {
    None,
    ZoomToScreen,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LaptopFrame {
    #[setting(
        "Screen (image or video)",
        default = "app:placeholders/screen-wide.svg",
        widget = "asset"
    )]
    pub screen_src: String,
    #[setting("Entrance", default = LaptopEntrance::Open)]
    pub entrance: LaptopEntrance,
    #[setting("Finale", default = LaptopFinale::ZoomToScreen)]
    pub finale: LaptopFinale,
    #[setting("Notch label", default = "Recording started")]
    pub notch_label: String,
    #[setting("Notch", default = true)]
    pub show_notch: bool,
    #[setting("Battery level", default = 85, range(min = 0, max = 100), step = 1)]
    pub battery_level: u32,
    #[setting("Bezel color", default = "#1D1D1F", check = colour)]
    pub bezel_color: String,
    #[setting(
        "Screen color",
        default = "#FFFFFF",
        widget = "color",
        zero_label = "Theme"
    )]
    pub screen_color: String,
    #[setting("Lid tilt", default = 18, range(min = 0, max = 45), step = 1)]
    pub rest_tilt: u32,
    #[setting("Lid radius", default = 22, range(min = 8, max = 40), step = 1)]
    pub radius: u32,
    #[setting("Scale", default = 0.85, range(min = 0.4, max = 1.4), step = 0.05)]
    pub scale: f64,
    #[setting("Float loop", default = true)]
    pub float_loop: bool,
    #[setting("Float amplitude", default = 4, range(min = 0, max = 16), step = 1)]
    pub float_amplitude: u32,
    #[setting("Shadow", default = "0 40px 80px rgba(10,12,20,0.45)")]
    pub shadow: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Recording light", default = "#30D158", check = colour)]
    pub indicator_color: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LaptopFrame>(
            element(
                ElementKind::Component,
                "laptop-frame",
                "Laptop frame",
                "A laptop that opens, shows a notification in its notch, then moves the camera into the screen until your clip fills the frame.",
                ElementCategory::Device,
                240,
            )
                .authored_clock()
                .design(1920, 1080)
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:laptop").moves(NodeMotion::Open)),
        )
}
