use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckListItem {
    #[default = "New item"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = true]
    #[schemars(description = "Ticked")]
    pub checked: bool,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckList {
    #[default = 40]
    #[schemars(description = "Font size", range(min = 24, max = 64), extend("x-step" = 2))]
    pub font_size: u32,
    #[default = 0.6]
    #[schemars(
        description = "Time between items",
        range(min = 0.2, max = 1.6),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub item_gap: f64,
    #[default = 0.3]
    #[schemars(
        description = "Time between ticks",
        range(min = 0.1, max = 0.8),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub close_gap: f64,
    #[default = 1.6]
    #[schemars(description = "Letters / pose", range(min = 0.8, max = 3), extend("x-step" = 0.2))]
    pub per_step: f64,
    #[default = 3]
    #[schemars(description = "Stroke width", range(min = 1, max = 8), extend("x-step" = 1))]
    pub stroke_width: u32,
    #[default = "#26242c"]
    #[schemars(
        description = "Label color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#6f7f35"]
    #[schemars(
        description = "Tick color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub tick_color: String,
    #[default = 0.1]
    #[schemars(
        description = "Time per pose",
        range(min = 0.0333, max = 0.2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub step: f64,
    #[default(vec![
        CheckListItem { text: "Record the walkthrough".into(), ..Default::default() },
        CheckListItem { text: "Add captions and zooms".into(), ..Default::default() },
        CheckListItem { text: "Export for the launch".into(), checked: false },
    ])]
    #[schemars(description = "Items")]
    pub items: Vec<CheckListItem>,
    #[default = 720]
    #[schemars(
        description = "Width",
        range(min = 200, max = 1800),
        extend("x-step" = 10, "x-unit" = "px")
    )]
    pub width: u32,
    #[default = "#26242c"]
    #[schemars(
        description = "Box colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub box_color: String,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub weight: FontWeight,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Row gap",
        range(min = 0, max = 200),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 1,
            "x-unit" = "px",
            "x-advanced" = true,
            "type" = "integer"
        )
    )]
    pub row_gap: Option<u32>,
    #[default = "checklist"]
    #[schemars(
        description = "Variation",
        extend(
            "x-help" = "Change it to draw the handwriting a little differently.",
            "x-advanced" = true
        )
    )]
    pub seed: String,
    #[default = "white"]
    #[schemars(
        description = "Background",
        extend(
            "x-help" = "Set it to transparent to lay the text over other layers.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
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
                .look(NodeLook::of("lucide:list-checks")),
        )
}
