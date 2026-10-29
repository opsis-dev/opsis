use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoWallLogo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Image",
        widget = "asset",
        accept = "image/*",
        zero_label = "Name and glyph",
        "type" = "string"
    )]
    pub image: Option<String>,
    #[setting("Name", default = "Marrow")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Glyph", "type" = "string")]
    pub glyph: Option<String>,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoWall {
    #[setting(
        "Logos",
        default = vec![
            LogoWallLogo {
                name: "Northstar".into(),
                color: Some("#E8B86D".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Halcyon".into(),
                color: Some("#7DD3E8".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Fernweh".into(),
                color: Some("#9BD4A0".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Orbital".into(),
                color: Some("#C99BE8".into()),
                ..Default::default()
            },
            LogoWallLogo { name: "Lumen".into(), color: Some("#8FB8F0".into()), ..Default::default() },
            LogoWallLogo {
                name: "Tidewater".into(),
                color: Some("#E89B9B".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Kestrel".into(),
                color: Some("#E8B86D".into()),
                ..Default::default()
            },
            LogoWallLogo { color: Some("#7DD3E8".into()), ..Default::default() },
        ],
        help = "Wordmark, brand colour and the glyph in the mark beside it, or a logo image in their place."
    )]
    pub logos: Vec<LogoWallLogo>,
    #[setting(
        "Small heading",
        default = "Trusted by",
        help = "Small line above the heading. Omit to drop it."
    )]
    pub eyebrow: String,
    #[setting(
        "Title",
        default = "Teams that ship with us",
        help = "Heading above the wall. Omit to drop it."
    )]
    pub title: String,
    #[setting(
        "Columns",
        default = 4,
        range(min = 1),
        step = 1,
        help = "Tiles per row."
    )]
    pub columns: u32,
    #[setting(
        "Start at (seconds)",
        default = 0.32,
        step = 0.01,
        help = "Second the first tile arrives.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Arrive stagger (seconds)",
        default = 0.06,
        step = 0.01,
        help = "Seconds between tiles arriving, in reading order. Deliberately fast.",
        advanced
    )]
    pub arrive_stagger_seconds: f64,
    #[setting(
        "Stagger (seconds)",
        default = 0.24,
        step = 0.01,
        help = "Seconds between diagonals of the colour sweep.",
        advanced
    )]
    pub stagger_seconds: f64,
    #[setting(
        "Colour fade (seconds)",
        default = 0.6,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "How long one tile takes to go from grey to brand colour.",
        advanced
    )]
    pub warm_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Eyebrow, and the fallback for logos with no colour.",
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
        "Mark text color",
        widget = "color",
        zero_label = "Theme",
        check = colour_or_empty,
        advanced
    )]
    pub glyph_color: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Wall width",
        default = 900,
        range(min = 300, max = 1800),
        step = 10,
        unit = "px",
        advanced
    )]
    pub wall_width: u32,
    #[setting(
        "Gap",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub gap: u32,
    #[setting(
        "Tile height",
        default = 88,
        range(min = 40, max = 240),
        step = 1,
        unit = "px",
        advanced
    )]
    pub tile_height: u32,
    #[setting(
        "Tile corners",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub tile_radius: u32,
    #[setting(
        "Title size",
        default = 30,
        range(min = 12, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub title_size: u32,
    #[setting("Uppercase small heading", default = true, advanced)]
    pub eyebrow_uppercase: bool,
    #[setting(
        "Name size",
        default = 19,
        range(min = 10, max = 48),
        step = 1,
        unit = "px",
        advanced
    )]
    pub name_size: u32,
    #[setting(
        "Logo image height",
        default = 46,
        range(min = 16, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub image_size: u32,
    #[setting(
        "Mark",
        default = true,
        help = "The small badge with the first letter or glyph.",
        advanced
    )]
    pub show_mark: bool,
    #[setting(
        "Starting brightness",
        default = 0.72,
        range(min = 0, max = 1),
        step = 0.01,
        help = "How dim the grey logos are before they warm up.",
        advanced
    )]
    pub start_brightness: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LogoWall>(
        element(
            ElementKind::Component,
            "logo-wall",
            "Logo wall",
            "A grid of customer logos that light up from grey to colour.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:layout-grid")),
    )
}
