use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

pub use crate::elements::overlay::keystroke_draw::draw;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, keystroke_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum KeyboardPlatform {
    Mac,
    Windows,
}

#[settings(export)]
pub struct KeystrokeStep {
    #[setting("Keys", default = "mod+k")]
    pub keys: String,
    #[setting(
        "At",
        range(min = 0, max = 120),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub at: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Hold",
        range(min = 0.0333, max = 20),
        zero_label = "Auto",
        step = 0.01,
        unit = "s",
        frames_at = 30,
        "type" = "number"
    )]
    pub hold: Option<f64>,
}

#[settings(export)]
pub struct Keystroke {
    #[setting("Platform", default = KeyboardPlatform::Mac)]
    pub platform: KeyboardPlatform,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Light),
        transform = keystroke_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Accent", default = "#0a84ff", check = colour)]
    pub accent: String,
    #[setting("Key size", default = 64, range(min = 32, max = 128), step = 1)]
    pub size: u32,
    #[setting(
        "Linger",
        default = 1.0,
        range(min = 0, max = 3),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub linger: f64,
    #[setting(
        "Where",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub x: f64,
    #[setting(
        "Where vertical",
        default = 0.86,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub y: f64,
    #[setting(
        "Keys",
        default = vec![
            KeystrokeStep { at: 0.6667, ..Default::default() },
            KeystrokeStep { at: 2.6, keys: "g i".into(), ..Default::default() },
            KeystrokeStep { at: 3.8, keys: "shift+mod+p".into(), ..Default::default() },
        ],
        help = "Join keys pressed together with +, like mod+k. Separate keys pressed one after another with a space."
    )]
    pub steps: Vec<KeystrokeStep>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting(
        "Font weight",
        default = 500,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting("Between keys", default = "then", advanced)]
    pub then_text: String,
    #[setting(
        "Hold",
        default = 0.2667,
        range(min = 0.0333, max = 20),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub hold: f64,
    #[setting("Key roundness", default = 0.16, range(min = 0, max = 0.5), advanced)]
    pub roundness: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Keystroke>(
        element(
            ElementKind::Component,
            "keystroke",
            "Keystroke",
            "Shows a keyboard shortcut as keycaps being pressed.",
            ElementCategory::Overlay,
            180,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:command")),
    )
}
