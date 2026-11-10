use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum LaptopEntrance {
    Rise,
    Open,
    None,
}

#[choice(export)]
pub enum LaptopFinale {
    #[label("None")]
    None,
    #[label("Zoom into the screen")]
    ZoomToScreen,
}

#[settings(export)]
pub struct LaptopFrame {
    #[setting("Entrance", default = LaptopEntrance::Open)]
    pub entrance: LaptopEntrance,
    #[setting("Finale", default = LaptopFinale::ZoomToScreen)]
    pub finale: LaptopFinale,
    #[setting("Notch label", default = "Recording started")]
    pub notch_label: String,
    #[setting("Notch", default = true)]
    pub show_notch: bool,
    #[setting(
        "Battery level",
        default = 85,
        range(min = 0, max = 100),
        step = 1,
        advanced
    )]
    pub battery_level: u32,
    #[setting("Bezel color", default = "#1D1D1F", check = colour)]
    pub bezel_color: String,
    #[setting(
        "Screen color",
        default = "#FFFFFF",
        widget = "color",
        zero_label = "Theme",
        advanced
    )]
    pub screen_color: String,
    #[setting("Lid tilt", default = 18, range(min = 0, max = 45), step = 1, advanced)]
    pub rest_tilt: u32,
    #[setting(
        "Lid radius",
        default = 22,
        range(min = 8, max = 40),
        step = 1,
        advanced
    )]
    pub radius: u32,
    #[setting("Scale", default = 0.85, range(min = 0.4, max = 1.4), step = 0.05)]
    pub scale: f64,
    #[setting("Float loop", default = true, advanced)]
    pub float_loop: bool,
    #[setting(
        "Float amplitude",
        default = 4,
        range(min = 0, max = 16),
        step = 1,
        advanced
    )]
    pub float_amplitude: u32,
    #[setting("Shadow", default = "0 40px 80px rgba(10,12,20,0.45)", advanced)]
    pub shadow: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Recording light", default = "#30D158", check = colour, advanced)]
    pub indicator_color: String,
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
        "Notch at",
        default = 4.0,
        range(min = 1.6, max = 20),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        help = "When the notch shows its label. It starts loading 1.6 s before.",
        advanced
    )]
    pub notch_at: f64,
    #[setting(
        "Zoom at",
        default = 4.6667,
        range(min = 0, max = 30),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        help = "When the finale zooms into the screen.",
        advanced
    )]
    pub zoom_at: f64,
    #[setting(
        "Zoom length",
        default = 1.5,
        range(min = 0.1, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub zoom_length: f64,
    #[setting(
        "Float period",
        default = 5.0,
        range(min = 1, max = 20),
        step = 0.1,
        unit = "s",
        show_if = { "floatLoop": ["true"] },
        advanced
    )]
    pub float_period: f64,
    #[setting(
        "Notch label size",
        default = 12,
        range(min = 8, max = 24),
        step = 1,
        unit = "px",
        advanced
    )]
    pub notch_label_size: u32,
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
                .look(NodeLook::of("lucide:laptop")),
        )
}
