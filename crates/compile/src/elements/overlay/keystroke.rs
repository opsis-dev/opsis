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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct KeystrokeStep {
    #[default = "mod+k"]
    #[schemars(description = "Keys")]
    pub keys: String,
    #[schemars(
        description = "At",
        range(min = 0, max = 120),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub at: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Hold",
        range(min = 0.0333, max = 20),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 0.01,
            "x-unit" = "s",
            "x-frames-at" = 30,
            "type" = "number"
        )
    )]
    pub hold: Option<f64>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Keystroke {
    #[default(KeyboardPlatform::Mac)]
    #[schemars(description = "Platform")]
    pub platform: KeyboardPlatform,
    #[default(LightOrDark::Light)]
    #[schemars(description = "Theme")]
    pub theme: LightOrDark,
    #[default = "#0a84ff"]
    #[schemars(
        description = "Accent",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent: String,
    #[default = 64]
    #[schemars(description = "Key size", range(min = 32, max = 128), extend("x-step" = 1))]
    pub size: u32,
    #[default = 1.0]
    #[schemars(
        description = "Linger",
        range(min = 0, max = 3),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub linger: f64,
    #[default = 0.5]
    #[schemars(
        description = "Where",
        range(min = 0, max = 1),
        extend(
            "x-step" = 0.01,
            "x-help" = "Click the frame to choose the spot.",
            "x-widget" = "spot"
        )
    )]
    pub x: f64,
    #[default = 0.86]
    #[schemars(
        description = "Where vertical",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-hidden" = true)
    )]
    pub y: f64,
    #[default(vec![
        KeystrokeStep { at: 0.6667, ..Default::default() },
        KeystrokeStep { at: 2.6, keys: "g i".into(), ..Default::default() },
        KeystrokeStep { at: 3.8, keys: "shift+mod+p".into(), ..Default::default() },
    ])]
    #[schemars(
        description = "Keys",
        extend(
            "x-help" = "Join keys pressed together with +, like mod+k. Separate keys pressed one after another with a space."
        )
    )]
    pub steps: Vec<KeystrokeStep>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
