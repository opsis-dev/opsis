use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct StatCard {
    #[default = 98.0]
    #[schemars(description = "Value", extend("x-help" = "Number the counter lands on."))]
    pub value: f64,
    #[default = "%"]
    #[schemars(description = "Suffix", extend("x-help" = "Unit after the number."))]
    pub suffix: String,
    #[schemars(description = "Prefix", extend("x-help" = "Unit before the number."))]
    pub prefix: String,
    #[schemars(
        description = "Decimals",
        range(min = 0, max = 6),
        extend(
            "x-step" = 1,
            "x-help" = "Decimal places while counting and at rest.",
            "x-advanced" = true
        )
    )]
    pub decimals: u32,
    #[default = "Satisfaction"]
    #[schemars(description = "Label", extend("x-help" = "Metric label."))]
    pub label: String,
    #[default = "Customer survey, last 30 days"]
    #[schemars(
        description = "Caption",
        extend("x-help" = "Line under the label (source, window, cohort).")
    )]
    pub caption: String,
    #[default = 4]
    #[schemars(
        description = "Change",
        range(min = -1000000, max = 1000000),
        extend(
            "x-step" = 1,
            "x-help" = "Change against the previous period; lands after the number settles.",
            "x-show-if" = { "showDelta": ["true"] }
        )
    )]
    pub delta: i32,
    #[default = "%"]
    #[schemars(
        description = "Change suffix",
        extend("x-help" = "Unit on the delta chip.", "x-show-if" = { "showDelta": ["true"] })
    )]
    pub delta_suffix: String,
    #[default = "#2DD4BF"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Ring, suffix, and delta colour.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-help" = "Animation speed multiplier.")
    )]
    pub speed: f64,
    #[default = true]
    #[schemars(description = "Show change")]
    pub show_delta: bool,
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Leave empty to follow the theme."
        )
    )]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Ring full at",
        range(min = 0, max = 1000000000),
        extend(
            "x-zero-label" = "Full ring",
            "x-step" = 1,
            "x-help" = "The ring fills to the value out of this. Leave empty for a full ring.",
            "type" = "integer"
        )
    )]
    pub max: Option<u32>,
}

pub fn definition() -> ElementMeta {
    meta::<StatCard>(
            element(
                ElementKind::Component,
                "stat-card",
                "Stat card",
                "One big number counts up while a ring fills, then its label and the change since last period appear.",
                ElementCategory::Data,
                123,
            )
                .look(NodeLook::of("lucide:circle-gauge")),
        )
}
