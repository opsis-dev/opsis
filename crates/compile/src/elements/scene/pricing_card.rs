use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct PricingCard {
    #[setting(
        "Plan name",
        default = "Studio",
        help = "Tier name across the top of the card."
    )]
    pub tier: String,
    #[setting("Price", default = 32.0, help = "The number the price rolls up to.")]
    pub price: f64,
    #[setting("Currency", default = "$", help = "Symbol set ahead of the number.")]
    pub currency: String,
    #[setting(
        "Billing period",
        default = "/mo",
        help = "Cadence after the price: \"/mo\", \"per seat\", anything short."
    )]
    pub period: String,
    #[setting(
        "Badge",
        default = "Most picked",
        help = "Chip beside the tier name. Omit to drop it."
    )]
    pub badge: String,
    #[setting(
        "Small print",
        default = "Billed annually. Cancel whenever.",
        help = "Line under the price."
    )]
    pub note: String,
    #[setting(
        "Features",
        default = vec![
            "2,000 render minutes a month".into(),
            "4K exports, no watermark".into(),
            "9 team seats".into(),
            "Render API and webhooks".into(),
            "Priority render queue".into(),
        ],
        help = "Ticked lines under the price, revealed in order."
    )]
    pub features: Vec<String>,
    #[setting(
        "Button label",
        default = "Start rendering",
        help = "Button text. Omit to drop the button."
    )]
    pub cta_label: String,
    #[setting(
        "Price at (seconds)",
        default = 0.42,
        step = 0.01,
        help = "Second the price starts rolling.",
        advanced
    )]
    pub price_at_seconds: f64,
    #[setting(
        "Roll (seconds)",
        default = 0.9,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "How long the roll takes.",
        advanced
    )]
    pub roll_seconds: f64,
    #[setting(
        "Features at (seconds)",
        default = 1.05,
        step = 0.1,
        help = "Second the first feature ticks in.",
        advanced
    )]
    pub features_at_seconds: f64,
    #[setting(
        "Feature stagger (seconds)",
        default = 0.22,
        step = 0.01,
        help = "Seconds between features. The CTA lands 0.16s after the last one.",
        advanced
    )]
    pub feature_stagger_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Badge, ticks, card border wash and the CTA fill.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = code_themes,
        widget = "theme"
    )]
    pub theme: Theme,
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
    #[setting(
        "Button text color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme.",
        check = colour_or_empty,
        advanced
    )]
    pub cta_text_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Old price",
        range(min = 0, max = 1000000),
        zero_label = "None",
        step = 1,
        help = "Shown struck out next to the price.",
        "type" = "integer"
    )]
    pub was_price: Option<u32>,
    #[setting(
        "Roll from",
        default = 0,
        range(min = 0, max = 1000000),
        step = 1,
        advanced
    )]
    pub roll_from: u32,
    #[setting(
        "Card width",
        default = 460,
        range(min = 300, max = 1000),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_width: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
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
