use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
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
        help = "Animation speed multiplier.",
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme.",
        advanced
    )]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Logo size",
        range(min = 10, max = 600),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        "type" = "integer",
        advanced
    )]
    pub logo_size: Option<u32>,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Accent glow",
        default = 12,
        range(min = 0, max = 60),
        step = 1,
        unit = "%",
        advanced
    )]
    pub glow: u32,
    #[setting(
        "Text size",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        help = "Scales every line of text together.",
        advanced
    )]
    pub text_size: f64,
    #[setting(
        "Title weight",
        default = 700,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub title_weight: u32,
    #[setting("Uppercase small heading", default = true, advanced)]
    pub eyebrow_caps: bool,
    #[setting(
        "Button corners",
        default = 999,
        range(min = 0, max = 999),
        step = 1,
        unit = "px",
        help = "999 makes a pill.",
        advanced
    )]
    pub button_radius: u32,
    #[setting("Button arrow", default = true, advanced)]
    pub button_arrow: bool,
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
        .look(NodeLook::of("lucide:flag")),
    )
}
