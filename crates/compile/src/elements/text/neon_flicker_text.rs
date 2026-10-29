use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum FlickerOrder {
    Start,
    End,
    Center,
    Edges,
    Random,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct NeonFlickerText {
    #[setting("Text", default = "Open all night", help = "The sign.")]
    pub text: String,
    #[setting(
        "Glow color",
        default = "#f472b6",
        help = "Colour of the halo around the tube.",
        check = colour
    )]
    pub glow_color: String,
    #[setting(
        "Glow size",
        default = 26.0,
        help = "Four stacked shadows, so it stays soft."
    )]
    pub glow_size: f64,
    #[setting(
        "Unlit brightness",
        default = 0.14,
        step = 0.01,
        help = "Brightness of an unlit tube, 0–1.",
        advanced
    )]
    pub off_level: f64,
    #[setting(
        "Hum",
        default = 0.12,
        step = 0.01,
        help = "Depth of the mains hum once the sign has settled.",
        advanced
    )]
    pub hum: f64,
    #[setting(
        "Buzz",
        default = 0.06,
        step = 0.01,
        help = "Chance per beat that a settled tube stutters. 0 disables it.",
        advanced
    )]
    pub buzz: f64,
    #[setting(
        "Flicker on exit",
        default = true,
        help = "Stutter back off instead of fading cleanly."
    )]
    pub flicker_on_exit: bool,
    #[setting("Order", default = FlickerOrder::Start, help = "Which tube strikes first.")]
    pub order: FlickerOrder,
    #[setting(
        "Stagger",
        default = 2,
        unit = "frames",
        help = "Frames between one tube striking and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Variation", default = 1, advanced)]
    pub seed: u32,
    #[setting("Color", default = "#fdf2f8", check = colour)]
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Font size",
        range(min = 12, max = 600),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        help = "Leave empty to size the text to the frame.",
        "type" = "integer"
    )]
    pub font_size: Option<u32>,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Unlit color",
        widget = "color",
        zero_label = "From the glow",
        help = "The tube with no gas lit: cold glass, not invisible. Leave empty for a faint glow colour."
    )]
    pub off_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<NeonFlickerText>(
        element(
            ElementKind::Component,
            "neon-flicker-text",
            "Neon sign",
            "Text that flickers on like a neon sign, then hums gently.",
            ElementCategory::Text,
            120,
        )
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Flicker)),
    )
}
