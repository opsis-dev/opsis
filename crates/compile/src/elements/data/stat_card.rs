use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::GroupSeparator;
use crate::elements::theme::{Theme, ThemeMode, stat_card_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct StatCard {
    #[setting("Value", default = 98.0, help = "Number the counter lands on.")]
    pub value: f64,
    #[setting("Suffix", default = "%", help = "Unit after the number.")]
    pub suffix: String,
    #[setting("Prefix", help = "Unit before the number.")]
    pub prefix: String,
    #[setting(
        "Decimals",
        range(min = 0, max = 6),
        step = 1,
        help = "Decimal places while counting and at rest.",
        advanced
    )]
    pub decimals: u32,
    #[setting("Label", default = "Satisfaction", help = "Metric label.")]
    pub label: String,
    #[setting(
        "Caption",
        default = "Customer survey, last 30 days",
        help = "Line under the label (source, window, cohort)."
    )]
    pub caption: String,
    #[setting(
        "Change",
        default = 4,
        range(min = -1000000, max = 1000000),
        step = 1,
        help = "Change against the previous period; lands after the number settles.",
        show_if = { "showDelta": ["true"] }
    )]
    pub delta: i32,
    #[setting(
        "Change suffix",
        default = "%",
        help = "Unit on the delta chip.",
        show_if = { "showDelta": ["true"] },
        advanced
    )]
    pub delta_suffix: String,
    #[setting(
        "Accent color",
        default = "#2DD4BF",
        help = "Ring, suffix, and delta colour.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = stat_card_themes,
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
    #[setting("Show change", default = true)]
    pub show_delta: bool,
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
        "Ring full at",
        range(min = 0, max = 1000000000),
        zero_label = "Full ring",
        step = 1,
        help = "The ring fills to the value out of this. Leave empty for a full ring.",
        "type" = "integer"
    )]
    pub max: Option<u32>,
    #[setting(
        "Glow",
        default = 10.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub glow: f64,
    #[setting(
        "Ring thickness",
        default = 9,
        range(min = 2, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub ring_thickness: u32,
    #[setting(
        "Thousands separator",
        default = GroupSeparator::None,
        advanced
    )]
    pub group_separator: GroupSeparator,
    #[setting("Font", widget = "font", zero_label = "Main font", advanced)]
    pub font_family: String,
    #[setting(
        "Falling colour",
        widget = "color",
        zero_label = "Accent",
        check = colour_or_empty,
        advanced
    )]
    pub down_color: String,
    #[setting(
        "Ring glow",
        default = 10,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub ring_glow: u32,
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
                .fonts(&["Inter"])
                .look(NodeLook::of("lucide:circle-gauge")),
        )
}
