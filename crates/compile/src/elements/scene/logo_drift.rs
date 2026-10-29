use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoDriftTile {
    #[setting("Mark", default = "Ab", help = "One to three characters.")]
    pub glyph: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Label", "type" = "string")]
    pub label: Option<String>,
    #[setting("Background", default = "#1f2430", check = colour)]
    pub background: String,
    #[setting("Text colour", widget = "color", zero_label = "Default")]
    pub color: String,
    #[setting("Start across", range(min = -1200, max = 1200), step = 1, advanced)]
    pub x: i32,
    #[setting("Start down", range(min = -800, max = 800), step = 1, advanced)]
    pub y: i32,
    #[setting("Drift across", range(min = -200, max = 200), step = 1, advanced)]
    pub vx: i32,
    #[setting("Drift down", range(min = -200, max = 200), step = 1, advanced)]
    pub vy: i32,
    #[setting("Size", default = 140, range(min = 20, max = 400), step = 1, advanced)]
    pub size: u32,
    #[setting(
        "Appears at",
        default = 1.0,
        range(min = 0, max = 10),
        step = 0.01,
        advanced
    )]
    pub at: f64,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoDrift {
    #[setting("Headline", default = "Built for the stack you already use.")]
    pub headline: String,
    #[setting(
        "Pull-back / sec",
        default = 0.1762,
        range(min = 0, max = 0.4),
        step = 0.001
    )]
    pub pullback: f64,
    #[setting("Drift", default = 1.0, range(min = 0, max = 3), step = 0.05)]
    pub tile_speed: f64,
    #[setting("Tile size", default = 1.0, range(min = 0.3, max = 2), step = 0.05)]
    pub tile_scale: f64,
    #[setting(
        "Tile fade τ (s)",
        default = 0.4,
        range(min = 0.05, max = 2),
        step = 0.01
    )]
    pub tile_fade: f64,
    #[setting("Tile radius", default = 0.02, range(min = 0, max = 0.5), step = 0.01)]
    pub tile_radius: f64,
    #[setting("Glyph size", default = 0.34, range(min = 0.1, max = 0.7), step = 0.01)]
    pub glyph_scale: f64,
    #[setting(
        "Word stagger (s)",
        default = 0.15,
        range(min = 0, max = 0.6),
        step = 0.005
    )]
    pub word_stagger: f64,
    #[setting(
        "Word lands in (s)",
        default = 0.14,
        range(min = 0.02, max = 1),
        step = 0.005
    )]
    pub word_duration: f64,
    #[setting(
        "Word arrives at",
        default = 1.216,
        range(min = 0.5, max = 2),
        step = 0.001
    )]
    pub word_scale: f64,
    #[setting("Word blur (px)", default = 4.3, range(min = 0, max = 20), step = 0.1)]
    pub word_blur: f64,
    #[setting("Exit at (s)", default = 3.13, range(min = 0.5, max = 10), step = 0.01)]
    pub exit_at: f64,
    #[setting(
        "Exit over (s)",
        default = 0.34,
        range(min = 0.05, max = 2),
        step = 0.01
    )]
    pub exit_duration: f64,
    #[setting("Headline size", default = 20.0, range(min = 8, max = 64), step = 0.5)]
    pub font_size: f64,
    #[setting("Wash", default = true)]
    pub glow: bool,
    #[setting(
        "Wash strength",
        default = 0.07,
        range(min = 0, max = 0.5),
        step = 0.005
    )]
    pub glow_opacity: f64,
    #[setting("Accent", widget = "color", zero_label = "Theme")]
    pub accent_color: String,
    #[setting("Theme", transform = warm_themes, widget = "theme")]
    pub theme: Theme,
    #[setting("Font weight", default = FontWeight::W500)]
    pub font_weight: FontWeight,
    #[setting("Glow colour", widget = "color", zero_label = "Accent")]
    pub glow_color: String,
    #[setting(
        "Tiles",
        default = vec![
            LogoDriftTile {
                glyph: "Rc".into(),
                label: Some("Record".into()),
                color: "#ffffff".into(),
                x: 163,
                y: 232,
                vx: -54,
                vy: 12,
                size: 109,
                at: 0.28,
                ..Default::default()
            },
            LogoDriftTile {
                glyph: "Ed".into(),
                label: Some("Edit".into()),
                background: "#f2f4f7".into(),
                color: "#16181d".into(),
                x: 128,
                y: -209,
                vx: 81,
                vy: 32,
                size: 122,
                at: 0.48,
            },
            LogoDriftTile {
                glyph: "Zm".into(),
                label: Some("Zoom".into()),
                background: "#10131a".into(),
                color: "#7dd3fc".into(),
                x: -384,
                y: 344,
                vx: -27,
                vy: -68,
                size: 200,
                at: 0.95,
            },
            LogoDriftTile {
                glyph: "Cc".into(),
                label: Some("Captions".into()),
                background: "#e7e8ea".into(),
                color: "#16181d".into(),
                x: 519,
                y: 45,
                vx: -31,
                vy: 64,
                size: 144,
                at: 1.51,
            },
            LogoDriftTile {
                glyph: "Mx".into(),
                label: Some("Mix".into()),
                background: "#f1ece2".into(),
                color: "#8a5a2b".into(),
                x: -692,
                y: -125,
                vx: 41,
                vy: -71,
                size: 214,
                at: 1.55,
            },
        ]
    )]
    pub tiles: Vec<LogoDriftTile>,
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
    meta::<LogoDrift>(
            element(
                ElementKind::Component,
                "logo-drift",
                "Logo drift",
                "Logo wall animation: a logo cloud of integrations drifting behind a headline. A field of tiles drifting past a headline while the camera pulls steadily back, so the wall of things-it-works-with keeps arriving from every edge.",
                ElementCategory::Scene,
                180,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:cloud")),
        )
}
