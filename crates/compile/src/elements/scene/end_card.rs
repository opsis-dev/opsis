use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct EndCard {
    #[setting("Title", default = "Ready to try it?", help = "Closing headline.")]
    pub title: String,
    #[setting(
        "Subtitle",
        default = "Start free. No credit card needed.",
        help = "Line under the title."
    )]
    pub subtitle: String,
    #[setting("Small heading", help = "Chip above the title.")]
    pub eyebrow: String,
    #[setting(
        "Button label",
        default = "Start free trial",
        help = "Button label. Omit to end on the title alone."
    )]
    pub cta: String,
    #[setting(
        "Address",
        default = "example.com/signup",
        help = "Address typed under the button."
    )]
    pub url: String,
    #[setting(
        "Handles",
        default = vec!["@acme".into(), "acme.example/blog".into()],
        help = "Handles or channels listed along the foot."
    )]
    pub handles: Vec<String>,
    #[setting("Logo", widget = "asset", accept = "image/*")]
    pub logo_src: String,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Button, pulse, and eyebrow colour.",
        check = colour
    )]
    pub accent_color: String,
    #[setting("Theme", default = DarkOrLight::Dark, help = "Surface palette.")]
    pub theme: DarkOrLight,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Logo size",
        range(min = 10, max = 600),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        "type" = "integer"
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
