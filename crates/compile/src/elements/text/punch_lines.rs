use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
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
pub struct PunchLines {
    #[setting(
        "Script (| card, / line)",
        default = "Record it. / Edit it. | One take. | Ship."
    )]
    pub script: String,
    #[setting("Card looks", default = "slide,punch")]
    pub styles: String,
    #[setting("Card sizes", default = "1,0.945,2.7")]
    pub sizes: String,
    #[setting("Card lengths", default = "1.6", frames_at = 30)]
    pub holds: String,
    #[setting("Accent card", default = 3, range(min = 0, max = 8), step = 1)]
    pub accent_beat: u32,
    #[setting("Font size", default = 93, range(min = 24, max = 200), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Line height (em)",
        default = 0.89,
        range(min = 0.7, max = 1.6),
        step = 0.005
    )]
    pub line_height: f64,
    #[setting("Letter spacing", default = "-0.022em")]
    pub letter_spacing: String,
    #[setting("Word spacing", default = "-0.02em")]
    pub word_spacing: String,
    #[setting("Ground", widget = "color", zero_label = "Theme")]
    pub ground: String,
    #[setting("Type", widget = "color", zero_label = "Theme")]
    pub ink: String,
    #[setting("Accent", default = "#ff1c8e", check = colour)]
    pub accent_color: String,
    #[setting(
        "Entrance lead",
        default = 0.1333,
        range(min = 0, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub lead: f64,
    #[setting("Slide from (em)", default = -0.737, range(min = -3, max = 0), step = 0.01)]
    pub slide_from: f64,
    #[setting(
        "Slide kick (em/s)",
        default = 7.34,
        range(min = 0, max = 20),
        step = 0.1
    )]
    pub slide_kick: f64,
    #[setting(
        "Slide settle",
        default = 2.846,
        range(min = 0.5, max = 8),
        step = 0.05
    )]
    pub slide_settle: f64,
    #[setting(
        "Line stagger",
        default = 0.1167,
        range(min = 0, max = 0.6667),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub line_stagger: f64,
    #[setting("Slide drop (em)", default = 0.313, range(min = -1, max = 1), step = 0.01)]
    pub slide_drop: f64,
    #[setting("Motion blur", default = 1.0, range(min = 0, max = 1), step = 0.05)]
    pub motion_blur: f64,
    #[setting("Punch from", default = 0.32, range(min = 0.05, max = 1), step = 0.01)]
    pub punch_from: f64,
    #[setting(
        "Punch",
        default = 0.5,
        range(min = 0.0667, max = 1.3333),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub punch_frames: f64,
    #[setting(
        "Word stagger",
        default = 0.0933,
        range(min = 0, max = 0.4),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub word_stagger: f64,
    #[setting("Push from", default = 0.778, range(min = 0.3, max = 1), step = 0.005)]
    pub push_from: f64,
    #[setting(
        "Push per second",
        default = 0.205,
        range(min = 0, max = 1),
        step = 0.005
    )]
    pub push_rate: f64,
    #[setting(
        "Rush lead",
        default = 0.4067,
        range(min = 0, max = 1.3333),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub rush_lead: f64,
    #[setting(
        "Rush",
        default = 0.6,
        range(min = 0.1333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub rush_frames: f64,
    #[setting(
        "Rush travel",
        default = 0.85,
        range(min = 0.2, max = 0.95),
        step = 0.01
    )]
    pub rush_travel: f64,
    #[setting("Fit width", default = 0.92, range(min = 0.4, max = 1), step = 0.01)]
    pub fit: f64,
    #[setting("Theme", transform = warm_themes, widget = "theme")]
    pub theme: Theme,
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
    meta::<PunchLines>(
            element(
                ElementKind::Component,
                "punch-lines",
                "Punch lines",
                "Kinetic typography title cards: bold statements that cut hard between beats. A run of full-frame title cards that cut hard between beats.",
                ElementCategory::Text,
                144,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:type").moves(NodeMotion::HardCut)),
        )
}
