use opsis_types::looks::NodeLook;
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
pub enum PhoneVariant {
    Flat,
    Tilt,
    Showcase,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum PhoneEntrance {
    Rise,
    RotateIn,
    Float,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct PhoneFrame {
    #[default(PhoneVariant::Showcase)]
    #[schemars(description = "Variant")]
    pub variant: PhoneVariant,
    #[default(PhoneEntrance::Float)]
    #[schemars(description = "Entrance")]
    pub entrance: PhoneEntrance,
    #[default = "app:placeholders/screen-tall.svg"]
    #[schemars(description = "Screen (image or video)", extend("x-widget" = "asset"))]
    pub screen_src: String,
    #[default = "#C7C9CE"]
    #[schemars(description = "Bezel color", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub bezel_color: String,
    #[default = "#0A0A0B"]
    #[schemars(
        description = "Screen color",
        extend("x-widget" = "color", "x-zero-label" = "Theme")
    )]
    pub screen_color: String,
    #[default = 53]
    #[schemars(description = "Body radius", range(min = 24, max = 90), extend("x-step" = 1))]
    pub radius: u32,
    #[default = 41]
    #[schemars(description = "Screen radius", range(min = 12, max = 78), extend("x-step" = 1))]
    pub screen_radius: u32,
    #[default = 0.92]
    #[schemars(description = "Scale", range(min = 0.4, max = 1.5), extend("x-step" = 0.05))]
    pub scale: f64,
    #[default(-12)]
    #[schemars(description = "Tilt angle", range(min = -30, max = 30), extend("x-step" = 1))]
    pub tilt_angle: i32,
    #[default = true]
    #[schemars(description = "Dynamic island")]
    pub show_dynamic_island: bool,
    #[schemars(description = "Float loop")]
    pub float_loop: bool,
    #[default = 6]
    #[schemars(description = "Float amplitude", range(min = 0, max = 20), extend("x-step" = 1))]
    pub float_amplitude: u32,
    #[default = "0 30px 70px rgba(16,24,40,0.28)"]
    #[schemars(description = "Shadow")]
    pub shadow: String,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
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
    meta::<PhoneFrame>(
            element(
                ElementKind::Component,
                "phone-frame",
                "Phone frame",
                "A phone frame with a notch and a screen slot for your clip, flat or tilted, with rise, rotate-in or float entrances.",
                ElementCategory::Device,
                240,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:smartphone")),
        )
}
