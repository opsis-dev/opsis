use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct PunchLines {
    #[default = "Record it. / Edit it. | One take. | Ship."]
    #[schemars(description = "Script (| card, / line)")]
    pub script: String,
    #[default = "slide,punch"]
    #[schemars(description = "Card looks")]
    pub styles: String,
    #[default = "1,0.945,2.7"]
    #[schemars(description = "Card sizes")]
    pub sizes: String,
    #[default = "1.6"]
    #[schemars(description = "Card lengths", extend("x-frames-at" = 30))]
    pub holds: String,
    #[default = 3]
    #[schemars(description = "Accent card", range(min = 0, max = 8), extend("x-step" = 1))]
    pub accent_beat: u32,
    #[default = 93]
    #[schemars(description = "Font size", range(min = 24, max = 200), extend("x-step" = 1))]
    pub font_size: u32,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = 0.89]
    #[schemars(
        description = "Line height (em)",
        range(min = 0.7, max = 1.6),
        extend("x-step" = 0.005)
    )]
    pub line_height: f64,
    #[default = "-0.022em"]
    #[schemars(description = "Letter spacing")]
    pub letter_spacing: String,
    #[default = "-0.02em"]
    #[schemars(description = "Word spacing")]
    pub word_spacing: String,
    #[schemars(description = "Ground", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub ground: String,
    #[schemars(description = "Type", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub ink: String,
    #[default = "#ff1c8e"]
    #[schemars(
        description = "Accent",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent_color: String,
    #[default = 0.1333]
    #[schemars(
        description = "Entrance lead",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub lead: f64,
    #[default(-0.737)]
    #[schemars(description = "Slide from (em)", range(min = -3, max = 0), extend("x-step" = 0.01))]
    pub slide_from: f64,
    #[default = 7.34]
    #[schemars(description = "Slide kick (em/s)", range(min = 0, max = 20), extend("x-step" = 0.1))]
    pub slide_kick: f64,
    #[default = 2.846]
    #[schemars(description = "Slide settle", range(min = 0.5, max = 8), extend("x-step" = 0.05))]
    pub slide_settle: f64,
    #[default = 0.1167]
    #[schemars(
        description = "Line stagger",
        range(min = 0, max = 0.6667),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub line_stagger: f64,
    #[default = 0.313]
    #[schemars(description = "Slide drop (em)", range(min = -1, max = 1), extend("x-step" = 0.01))]
    pub slide_drop: f64,
    #[default = 1.0]
    #[schemars(description = "Motion blur", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub motion_blur: f64,
    #[default = 0.32]
    #[schemars(description = "Punch from", range(min = 0.05, max = 1), extend("x-step" = 0.01))]
    pub punch_from: f64,
    #[default = 0.5]
    #[schemars(
        description = "Punch",
        range(min = 0.0667, max = 1.3333),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub punch_frames: f64,
    #[default = 0.0933]
    #[schemars(
        description = "Word stagger",
        range(min = 0, max = 0.4),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub word_stagger: f64,
    #[default = 0.778]
    #[schemars(description = "Push from", range(min = 0.3, max = 1), extend("x-step" = 0.005))]
    pub push_from: f64,
    #[default = 0.205]
    #[schemars(description = "Push per second", range(min = 0, max = 1), extend("x-step" = 0.005))]
    pub push_rate: f64,
    #[default = 0.4067]
    #[schemars(
        description = "Rush lead",
        range(min = 0, max = 1.3333),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub rush_lead: f64,
    #[default = 0.6]
    #[schemars(
        description = "Rush",
        range(min = 0.1333, max = 2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub rush_frames: f64,
    #[default = 0.85]
    #[schemars(description = "Rush travel", range(min = 0.2, max = 0.95), extend("x-step" = 0.01))]
    pub rush_travel: f64,
    #[default = 0.92]
    #[schemars(description = "Fit width", range(min = 0.4, max = 1), extend("x-step" = 0.01))]
    pub fit: f64,
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
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
                .look(NodeLook::of("lucide:type").moves(NodeMotion::HardCut)),
        )
}
