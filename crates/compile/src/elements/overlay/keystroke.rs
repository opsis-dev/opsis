use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::LightOrDark;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum KeyboardPlatform {
    Mac,
    Windows,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Keystroke {
    #[setting("Platform", default = KeyboardPlatform::Mac)]
    pub platform: KeyboardPlatform,
    #[setting("Theme", default = LightOrDark::Light)]
    pub theme: LightOrDark,
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
        frames_at = 30
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
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
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
