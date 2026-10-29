use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckListItem {
    #[setting("Text", default = "New item")]
    pub text: String,
    #[setting("Ticked", default = true)]
    pub checked: bool,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckList {
    #[setting("Font size", default = 40, range(min = 24, max = 64), step = 2)]
    pub font_size: u32,
    #[setting(
        "Time between items",
        default = 0.6,
        range(min = 0.2, max = 1.6),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub item_gap: f64,
    #[setting(
        "Time between ticks",
        default = 0.3,
        range(min = 0.1, max = 0.8),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub close_gap: f64,
    #[setting("Letters / pose", default = 1.6, range(min = 0.8, max = 3), step = 0.2)]
    pub per_step: f64,
    #[setting("Stroke width", default = 3, range(min = 1, max = 8), step = 1)]
    pub stroke_width: u32,
    #[setting("Label color", default = "#26242c", check = colour)]
    pub color: String,
    #[setting("Tick color", default = "#6f7f35", check = colour)]
    pub tick_color: String,
    #[setting(
        "Time per pose",
        default = 0.1,
        range(min = 0.0333, max = 0.2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub step: f64,
    #[setting(
        "Items",
        default = vec![
            CheckListItem { text: "Record the walkthrough".into(), ..Default::default() },
            CheckListItem { text: "Add captions and zooms".into(), ..Default::default() },
            CheckListItem { text: "Export for the launch".into(), checked: false },
        ]
    )]
    pub items: Vec<CheckListItem>,
    #[setting(
        "Width",
        default = 720,
        range(min = 200, max = 1800),
        step = 10,
        unit = "px"
    )]
    pub width: u32,
    #[setting("Box colour", default = "#26242c", check = colour)]
    pub box_color: String,
    #[setting("Font weight", default = FontWeight::W600)]
    pub weight: FontWeight,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Row gap",
        range(min = 0, max = 200),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub row_gap: Option<u32>,
    #[setting(
        "Variation",
        default = "checklist",
        help = "Change it to draw the handwriting a little differently.",
        advanced
    )]
    pub seed: String,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<CheckList>(
            element(
                ElementKind::Component,
                "check-list",
                "Check list",
                "A handwritten checklist that writes itself out in full, then ticks and strikes through every done item, one at a time.",
                ElementCategory::Ui,
                360,
            )
                .authored_clock()
                .fonts(&["Caveat"])
                .look(NodeLook::of("lucide:list-checks")),
        )
}
