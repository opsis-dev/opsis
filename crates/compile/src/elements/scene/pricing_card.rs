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
pub struct PricingCard {
    #[default = "Studio"]
    #[schemars(
        description = "Plan name",
        extend("x-help" = "Tier name across the top of the card.")
    )]
    pub tier: String,
    #[default = 32.0]
    #[schemars(description = "Price", extend("x-help" = "The number the price rolls up to."))]
    pub price: f64,
    #[default = "$"]
    #[schemars(description = "Currency", extend("x-help" = "Symbol set ahead of the number."))]
    pub currency: String,
    #[default = "/mo"]
    #[schemars(
        description = "Billing period",
        extend("x-help" = "Cadence after the price: \"/mo\", \"per seat\", anything short.")
    )]
    pub period: String,
    #[default = "Most picked"]
    #[schemars(
        description = "Badge",
        extend("x-help" = "Chip beside the tier name. Omit to drop it.")
    )]
    pub badge: String,
    #[default = "Billed annually. Cancel whenever."]
    #[schemars(description = "Small print", extend("x-help" = "Line under the price."))]
    pub note: String,
    #[default(vec![
        "2,000 render minutes a month".into(),
        "4K exports, no watermark".into(),
        "9 team seats".into(),
        "Render API and webhooks".into(),
        "Priority render queue".into(),
    ])]
    #[schemars(
        description = "Features",
        extend("x-help" = "Ticked lines under the price, revealed in order.")
    )]
    pub features: Vec<String>,
    #[default = "Start rendering"]
    #[schemars(
        description = "Button label",
        extend("x-help" = "Button text. Omit to drop the button.")
    )]
    pub cta_label: String,
    #[default = 0.42]
    #[schemars(
        description = "Price at (seconds)",
        extend("x-step" = 0.01, "x-help" = "Second the price starts rolling.", "x-advanced" = true)
    )]
    pub price_at_seconds: f64,
    #[default = 0.9]
    #[schemars(
        description = "Roll (seconds)",
        range(min = 0.05, max = 10),
        extend("x-step" = 0.01, "x-help" = "How long the roll takes.", "x-advanced" = true)
    )]
    pub roll_seconds: f64,
    #[default = 1.05]
    #[schemars(
        description = "Features at (seconds)",
        extend("x-step" = 0.1, "x-help" = "Second the first feature ticks in.", "x-advanced" = true)
    )]
    pub features_at_seconds: f64,
    #[default = 0.22]
    #[schemars(
        description = "Feature stagger (seconds)",
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between features. The CTA lands 0.16s after the last one.",
            "x-advanced" = true
        )
    )]
    pub feature_stagger_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Badge, ticks, card border wash and the CTA fill.",
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
        description = "Old price",
        range(min = 0, max = 1000000),
        extend(
            "x-zero-label" = "None",
            "x-step" = 1,
            "x-help" = "Shown struck out next to the price.",
            "type" = "integer"
        )
    )]
    pub was_price: Option<u32>,
}

pub fn definition() -> ElementMeta {
    meta::<PricingCard>(
        element(
            ElementKind::Component,
            "pricing-card",
            "Pricing card",
            "A pricing plan card: the price rolls in, then each feature is ticked off.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:badge-dollar-sign")),
    )
}
