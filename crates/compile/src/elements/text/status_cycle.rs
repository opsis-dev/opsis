use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, FontWeight};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct StatusCycle {
    #[default = "Acme is"]
    #[schemars(description = "Prefix")]
    pub prefix: String,
    #[default = "recording, editing, exporting, live"]
    #[schemars(description = "Statuses (comma-separated)")]
    pub statuses: String,
    #[default = "Record, Auto zoom, Captions, Export, Share"]
    #[schemars(description = "Chips (comma-separated)")]
    pub chips: String,
    #[schemars(
        description = "Field (act 1)",
        extend("x-widget" = "color", "x-zero-label" = "Theme")
    )]
    pub field_color: String,
    #[schemars(
        description = "Page (act 2)",
        extend("x-widget" = "color", "x-zero-label" = "Theme")
    )]
    pub page_color: String,
    #[schemars(description = "Pill", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub pill_color: String,
    #[schemars(
        description = "Pill label",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub pill_label_color: String,
    #[schemars(description = "Prefix ink", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub prefix_color: String,
    #[schemars(
        description = "Chip fills",
        length(min = 0, max = 8),
        extend("x-help" = "Leave empty for the theme's chip colours.")
    )]
    pub chip_fills: Vec<Colour>,
    #[schemars(description = "Chip ink", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub chip_text_color: String,
    #[default = "rgb(160, 160, 160)"]
    #[schemars(
        description = "Chip hairline",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub chip_border_color: String,
    #[default(FontWeight::W400)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default(FontWeight::W400)]
    #[schemars(description = "Chip font weight")]
    pub chip_font_weight: FontWeight,
    #[default = 63]
    #[schemars(description = "Status size", range(min = 24, max = 140), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = 160]
    #[schemars(description = "Chip size", range(min = 48, max = 260), extend("x-step" = 2))]
    pub chip_font_size: u32,
    #[default = 0.8]
    #[schemars(
        description = "Intro length",
        range(min = 0.2667, max = 2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub intro_frames: f64,
    #[default = 1.0]
    #[schemars(
        description = "Status hold",
        range(min = 0.3333, max = 2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub status_hold: f64,
    #[default = 0.2667]
    #[schemars(
        description = "Chip stagger",
        range(min = 0.1333, max = 1),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub chip_stagger: f64,
    #[default = 0.516]
    #[schemars(
        description = "Lockup height",
        range(min = 0.2, max = 0.8),
        extend("x-step" = 0.004)
    )]
    pub pill_center_y: f64,
    #[schemars(
        description = "Start at",
        range(min = 0, max = 2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub start_at: f64,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Status corner radius",
        range(min = 0, max = 200),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 1,
            "x-unit" = "px",
            "x-advanced" = true,
            "type" = "integer"
        )
    )]
    pub pill_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Chip corner radius",
        range(min = 0, max = 200),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 1,
            "x-unit" = "px",
            "x-advanced" = true,
            "type" = "integer"
        )
    )]
    pub chip_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Chip border width",
        range(min = 0, max = 20),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 0.5,
            "x-unit" = "px",
            "x-advanced" = true,
            "type" = "number"
        )
    )]
    pub chip_border_width: Option<f64>,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
