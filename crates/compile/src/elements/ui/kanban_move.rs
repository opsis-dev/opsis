use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct KanbanMoveCard {
    #[default = "CSV export"]
    #[schemars(description = "Title")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Meta", extend("type" = "string"))]
    pub meta: Option<String>,
    #[default = 2]
    #[schemars(description = "Column")]
    pub column: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Tint",
        extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string")
    )]
    pub tint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Move to", extend("type" = "integer"))]
    pub move_to: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Move at seconds", extend("type" = "number"))]
    pub move_at_seconds: Option<f64>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct KanbanMove {
    #[default(vec!["Backlog".into(), "In progress".into(), "Shipped".into()])]
    #[schemars(
        description = "Columns",
        extend("x-help" = "Column headings, left to right. Cards address columns by index.")
    )]
    pub columns: Vec<String>,
    #[default(vec![
        KanbanMoveCard {
            title: "Onboarding checklist".into(),
            meta: Some("APP-218 · Ana".into()),
            column: 0,
            tint: Some("#8FB8F0".into()),
            ..Default::default()
        },
        KanbanMoveCard {
            title: "Retry failed payments".into(),
            meta: Some("APP-204 · Piotr".into()),
            column: 0,
            tint: Some("#E8B86D".into()),
            move_to: Some(1),
            move_at_seconds: Some(0.95),
        },
        KanbanMoveCard {
            title: "Dark mode".into(),
            meta: Some("APP-231 · Dai".into()),
            column: 0,
            tint: Some("#C99BE8".into()),
            ..Default::default()
        },
        KanbanMoveCard {
            title: "Search filters".into(),
            meta: Some("APP-190 · Ana".into()),
            column: 1,
            tint: Some("#7DD3E8".into()),
            move_to: Some(2),
            move_at_seconds: Some(1.72),
        },
        KanbanMoveCard {
            title: "Team invites".into(),
            meta: Some("APP-177 · Kit".into()),
            column: 1,
            tint: Some("#9BD4A0".into()),
            ..Default::default()
        },
        KanbanMoveCard {
            meta: Some("APP-166 · Dai".into()),
            tint: Some("#9BD4A0".into()),
            ..Default::default()
        },
    ])]
    #[schemars(
        description = "Cards",
        extend(
            "x-help" = "Title, meta line, starting column, rail tint, and the one move the card makes."
        )
    )]
    pub cards: Vec<KanbanMoveCard>,
    #[default = 0.62]
    #[schemars(
        description = "Move time (seconds)",
        range(min = 0.05, max = 10),
        extend(
            "x-step" = 0.01,
            "x-help" = "How long a card takes to arc from column to column.",
            "x-advanced" = true
        )
    )]
    pub move_seconds: f64,
    #[default = 0.08]
    #[schemars(
        description = "Card stagger (seconds)",
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between each card's arrival during the opening deal.",
            "x-advanced" = true
        )
    )]
    pub deal_stagger_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Destination column glow, and the fallback card rail.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-help" = "Animation speed multiplier.")
    )]
    pub speed: f64,
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Leave empty to follow the theme."
        )
    )]
    pub background_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<KanbanMove>(
        element(
            ElementKind::Component,
            "kanban-move",
            "Kanban board",
            "A task board where cards move from one column to the next.",
            ElementCategory::Ui,
            120,
        )
        .look(NodeLook::of("lucide:square-kanban")),
    )
}
