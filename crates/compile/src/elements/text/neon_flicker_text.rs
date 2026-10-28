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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct NeonFlickerText {
    #[default = "Open all night"]
    #[schemars(description = "Text", extend("x-help" = "The sign."))]
    pub text: String,
    #[default = "#f472b6"]
    #[schemars(
        description = "Glow color",
        extend(
            "x-help" = "Colour of the halo around the tube.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub glow_color: String,
    #[default = 26.0]
    #[schemars(
        description = "Glow size",
        extend("x-help" = "Four stacked shadows, so it stays soft.")
    )]
    pub glow_size: f64,
    #[default = 0.14]
    #[schemars(
        description = "Unlit brightness",
        extend("x-step" = 0.01, "x-help" = "Brightness of an unlit tube, 0–1.", "x-advanced" = true)
    )]
    pub off_level: f64,
    #[default = 0.12]
    #[schemars(
        description = "Hum",
        extend(
            "x-step" = 0.01,
            "x-help" = "Depth of the mains hum once the sign has settled.",
            "x-advanced" = true
        )
    )]
    pub hum: f64,
    #[default = 0.06]
    #[schemars(
        description = "Buzz",
        extend(
            "x-step" = 0.01,
            "x-help" = "Chance per beat that a settled tube stutters. 0 disables it.",
            "x-advanced" = true
        )
    )]
    pub buzz: f64,
    #[default = true]
    #[schemars(
        description = "Flicker on exit",
        extend("x-help" = "Stutter back off instead of fading cleanly.")
    )]
    pub flicker_on_exit: bool,
    #[default(FlickerOrder::Start)]
    #[schemars(description = "Order", extend("x-help" = "Which tube strikes first."))]
    pub order: FlickerOrder,
    #[default = 2]
    #[schemars(
        description = "Stagger",
        extend(
            "x-unit" = "frames",
            "x-help" = "Frames between one tube striking and the next.",
            "x-advanced" = true
        )
    )]
    pub stagger_in_frames: u32,
    #[default = 1]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: u32,
    #[default = "#fdf2f8"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Font size",
        range(min = 12, max = 600),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 1,
            "x-unit" = "px",
            "x-help" = "Leave empty to size the text to the frame.",
            "type" = "integer"
        )
    )]
    pub font_size: Option<u32>,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[schemars(
        description = "Unlit color",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "From the glow",
            "x-help" = "The tube with no gas lit: cold glass, not invisible. Leave empty for a faint glow colour."
        )
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
