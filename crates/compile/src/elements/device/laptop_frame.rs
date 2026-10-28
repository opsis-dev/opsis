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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LaptopFrame {
    #[default = "app:placeholders/screen-wide.svg"]
    #[schemars(description = "Screen (image or video)", extend("x-widget" = "asset"))]
    pub screen_src: String,
    #[default(LaptopEntrance::Open)]
    #[schemars(description = "Entrance")]
    pub entrance: LaptopEntrance,
    #[default(LaptopFinale::ZoomToScreen)]
    #[schemars(description = "Finale")]
    pub finale: LaptopFinale,
    #[default = "Recording started"]
    #[schemars(description = "Notch label")]
    pub notch_label: String,
    #[default = true]
    #[schemars(description = "Notch")]
    pub show_notch: bool,
    #[default = 85]
    #[schemars(description = "Battery level", range(min = 0, max = 100), extend("x-step" = 1))]
    pub battery_level: u32,
    #[default = "#1D1D1F"]
    #[schemars(
        description = "Bezel color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub bezel_color: String,
    #[default = "#FFFFFF"]
    #[schemars(
        description = "Screen color",
        extend("x-widget" = "color", "x-zero-label" = "Theme")
    )]
    pub screen_color: String,
    #[default = 18]
    #[schemars(description = "Lid tilt", range(min = 0, max = 45), extend("x-step" = 1))]
    pub rest_tilt: u32,
    #[default = 22]
    #[schemars(description = "Lid radius", range(min = 8, max = 40), extend("x-step" = 1))]
    pub radius: u32,
    #[default = 0.85]
    #[schemars(description = "Scale", range(min = 0.4, max = 1.4), extend("x-step" = 0.05))]
    pub scale: f64,
    #[default = true]
    #[schemars(description = "Float loop")]
    pub float_loop: bool,
    #[default = 4]
    #[schemars(description = "Float amplitude", range(min = 0, max = 16), extend("x-step" = 1))]
    pub float_amplitude: u32,
    #[default = "0 40px 80px rgba(10,12,20,0.45)"]
    #[schemars(description = "Shadow")]
    pub shadow: String,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default = "#30D158"]
    #[schemars(
        description = "Recording light",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub indicator_color: String,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
