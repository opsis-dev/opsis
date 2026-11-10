use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum PhoneVariant {
    #[label("Straight on")]
    Flat,
    #[label("Turned")]
    Tilt,
}

#[choice(export)]
pub enum PhoneEntrance {
    #[label("Slide in")]
    Rise,
    #[label("Turn in")]
    RotateIn,
    #[label("Float")]
    Float,
}

#[settings(export)]
pub struct PhoneFrame {
    #[setting("Variant", default = PhoneVariant::Flat)]
    pub variant: PhoneVariant,
    #[setting("Entrance", default = PhoneEntrance::Float)]
    pub entrance: PhoneEntrance,
    #[setting(
        "Bezel color",
        default = "#C7C9CE",
        widget = "color",
        zero_label = "Theme"
    )]
    pub bezel_color: String,
    #[setting(
        "Screen color",
        default = "#0A0A0B",
        widget = "color",
        zero_label = "Theme",
        advanced
    )]
    pub screen_color: String,
    #[setting(
        "Body radius",
        default = 53,
        range(min = 24, max = 90),
        step = 1,
        advanced
    )]
    pub radius: u32,
    #[setting(
        "Screen radius",
        default = 41,
        range(min = 12, max = 78),
        step = 1,
        advanced
    )]
    pub screen_radius: u32,
    #[setting("Scale", default = 0.92, range(min = 0.4, max = 1.5), step = 0.05)]
    pub scale: f64,
    #[setting("Tilt angle", default = -12, range(min = -30, max = 30), step = 1, advanced)]
    pub tilt_angle: i32,
    #[setting("Dynamic island", default = true)]
    pub show_dynamic_island: bool,
    #[setting("Float loop")]
    pub float_loop: bool,
    #[setting(
        "Float amplitude",
        default = 6,
        range(min = 0, max = 20),
        step = 1,
        advanced
    )]
    pub float_amplitude: u32,
    #[setting("Shadow", default = "0 30px 70px rgba(16,24,40,0.28)", advanced)]
    pub shadow: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Float period",
        default = 4.5,
        range(min = 1, max = 20),
        step = 0.1,
        unit = "s",
        show_if = { "floatLoop": ["true"] },
        advanced
    )]
    pub float_period: f64,
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
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:smartphone")),
        )
}
