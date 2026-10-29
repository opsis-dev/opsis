use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, FontWeight};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct StatusCycle {
    #[setting("Prefix", default = "Acme is")]
    pub prefix: String,
    #[setting(
        "Statuses (comma-separated)",
        default = "recording, editing, exporting, live"
    )]
    pub statuses: String,
    #[setting(
        "Chips (comma-separated)",
        default = "Record, Auto zoom, Captions, Export, Share"
    )]
    pub chips: String,
    #[setting("Field (act 1)", widget = "color", zero_label = "Theme")]
    pub field_color: String,
    #[setting("Page (act 2)", widget = "color", zero_label = "Theme")]
    pub page_color: String,
    #[setting("Pill", widget = "color", zero_label = "Theme")]
    pub pill_color: String,
    #[setting("Pill label", widget = "color", zero_label = "Default")]
    pub pill_label_color: String,
    #[setting("Prefix ink", widget = "color", zero_label = "Theme")]
    pub prefix_color: String,
    #[setting(
        "Chip fills",
        length(min = 0, max = 8),
        help = "Leave empty for the theme's chip colours."
    )]
    pub chip_fills: Vec<Colour>,
    #[setting("Chip ink", widget = "color", zero_label = "Theme")]
    pub chip_text_color: String,
    #[setting("Chip hairline", default = "rgb(160, 160, 160)", check = colour)]
    pub chip_border_color: String,
    #[setting("Font weight", default = FontWeight::W400)]
    pub font_weight: FontWeight,
    #[setting("Chip font weight", default = FontWeight::W400)]
    pub chip_font_weight: FontWeight,
    #[setting("Status size", default = 63, range(min = 24, max = 140), step = 1)]
    pub font_size: u32,
    #[setting("Chip size", default = 160, range(min = 48, max = 260), step = 2)]
    pub chip_font_size: u32,
    #[setting(
        "Intro length",
        default = 0.8,
        range(min = 0.2667, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub intro_frames: f64,
    #[setting(
        "Status hold",
        default = 1.0,
        range(min = 0.3333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub status_hold: f64,
    #[setting(
        "Chip stagger",
        default = 0.2667,
        range(min = 0.1333, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub chip_stagger: f64,
    #[setting(
        "Lockup height",
        default = 0.516,
        range(min = 0.2, max = 0.8),
        step = 0.004
    )]
    pub pill_center_y: f64,
    #[setting(
        "Start at",
        range(min = 0, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub start_at: f64,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Status corner radius",
        range(min = 0, max = 200),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub pill_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Chip corner radius",
        range(min = 0, max = 200),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub chip_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Chip border width",
        range(min = 0, max = 20),
        zero_label = "Auto",
        step = 0.5,
        unit = "px",
        advanced,
        "type" = "number"
    )]
    pub chip_border_width: Option<f64>,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<StatusCycle>(
            element(
                ElementKind::Component,
                "status-cycle",
                "Status cycle",
                "Status badge animation: a pill whose label cycles through states. A centred line whose status pill cycles: the label rolls up behind a hard clip while the pill's width springs past its target and back, and the line re-centres around it.",
                ElementCategory::Text,
                198,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:badge")),
        )
}
