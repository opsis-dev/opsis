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
pub struct KanbanMoveCard {
    #[setting("Title", default = "CSV export")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Meta", "type" = "string")]
    pub meta: Option<String>,
    #[setting("Column", default = 2)]
    pub column: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Tint", widget = "color", zero_label = "None", "type" = "string")]
    pub tint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Move to", "type" = "integer")]
    pub move_to: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Move at seconds", "type" = "number")]
    pub move_at_seconds: Option<f64>,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct KanbanMove {
    #[setting(
        "Columns",
        default = vec!["Backlog".into(), "In progress".into(), "Shipped".into()],
        help = "Column headings, left to right. Cards address columns by index."
    )]
    pub columns: Vec<String>,
    #[setting(
        "Cards",
        default = vec![
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
        ],
        help = "Title, meta line, starting column, rail tint, and the one move the card makes."
    )]
    pub cards: Vec<KanbanMoveCard>,
    #[setting(
        "Move time (seconds)",
        default = 0.62,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "How long a card takes to arc from column to column.",
        advanced
    )]
    pub move_seconds: f64,
    #[setting(
        "Card stagger (seconds)",
        default = 0.08,
        step = 0.01,
        help = "Seconds between each card's arrival during the opening deal.",
        advanced
    )]
    pub deal_stagger_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Destination column glow, and the fallback card rail.",
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
    #[setting("Show counts", default = true, advanced)]
    pub show_counts: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
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
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:square-kanban")),
    )
}
