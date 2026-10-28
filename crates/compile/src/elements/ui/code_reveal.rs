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
pub struct CodeReveal {
    #[default = "export async function checkout(cart: Cart) {\n  const total = cart.items.reduce((sum, item) => sum + item.price, 0);\n  const order = await api.orders.create({ items: cart.items, total });\n\n  return order.id;\n}"]
    #[schemars(
        description = "Code",
        extend(
            "x-widget" = "code",
            "x-help" = "Source shown in the editor; surrounding blank lines are trimmed."
        )
    )]
    pub code: String,
    #[default(vec![3.0])]
    #[schemars(
        description = "Highlighted lines",
        extend(
            "x-help" = "Line numbers as shown in the gutter, focused once the listing finishes writing."
        )
    )]
    pub highlighted_lines: Vec<f64>,
    #[default = "checkout.ts"]
    #[schemars(description = "Title", extend("x-help" = "Filename on the editor tab."))]
    pub title: String,
    #[default = "tsx"]
    #[schemars(
        description = "Language",
        extend("x-help" = "Language badge on the right of the header.", "x-advanced" = true)
    )]
    pub language: String,
    #[default = 1]
    #[schemars(
        description = "Start line",
        extend("x-help" = "First line number in the gutter, for excerpts.", "x-advanced" = true)
    )]
    pub start_line: u32,
    #[default = true]
    #[schemars(description = "Line numbers", extend("x-help" = "Shows the gutter."))]
    pub show_line_numbers: bool,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Caret, focus band and glow color.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Editor palette."))]
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
    meta::<CodeReveal>(
        element(
            ElementKind::Component,
            "code-reveal",
            "Code reveal",
            "A code editor that types out a file, then highlights the lines that matter.",
            ElementCategory::Ui,
            165,
        )
        .look(NodeLook::of("lucide:code")),
    )
}
