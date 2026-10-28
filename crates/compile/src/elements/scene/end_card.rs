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
pub struct EndCard {
    #[default = "Ready to try it?"]
    #[schemars(description = "Title", extend("x-help" = "Closing headline."))]
    pub title: String,
    #[default = "Start free. No credit card needed."]
    #[schemars(description = "Subtitle", extend("x-help" = "Line under the title."))]
    pub subtitle: String,
    #[schemars(description = "Small heading", extend("x-help" = "Chip above the title."))]
    pub eyebrow: String,
    #[default = "Start free trial"]
    #[schemars(
        description = "Button label",
        extend("x-help" = "Button label. Omit to end on the title alone.")
    )]
    pub cta: String,
    #[default = "example.com/signup"]
    #[schemars(description = "Address", extend("x-help" = "Address typed under the button."))]
    pub url: String,
    #[default(vec!["@acme".into(), "acme.example/blog".into()])]
    #[schemars(
        description = "Handles",
        extend("x-help" = "Handles or channels listed along the foot.")
    )]
    pub handles: Vec<String>,
    #[schemars(description = "Logo", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub logo_src: String,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Button, pulse, and eyebrow colour.",
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
        description = "Logo size",
        range(min = 10, max = 600),
        extend("x-zero-label" = "Auto", "x-step" = 1, "x-unit" = "px", "type" = "integer")
    )]
    pub logo_size: Option<u32>,
}

pub fn definition() -> ElementMeta {
    meta::<EndCard>(
        element(
            ElementKind::Component,
            "end-card",
            "End card",
            "A closing card with a title, a button, your web address and social handles.",
            ElementCategory::Scene,
            150,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:flag")),
    )
}
