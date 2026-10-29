use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CodeReveal {
    #[setting(
        "Code",
        default = "export async function checkout(cart: Cart) {\n  const total = cart.items.reduce((sum, item) => sum + item.price, 0);\n  const order = await api.orders.create({ items: cart.items, total });\n\n  return order.id;\n}",
        widget = "code",
        help = "Source shown in the editor; surrounding blank lines are trimmed."
    )]
    pub code: String,
    #[setting(
        "Highlighted lines",
        default = vec![3.0],
        help = "Line numbers as shown in the gutter, focused once the listing finishes writing."
    )]
    pub highlighted_lines: Vec<f64>,
    #[setting("Title", default = "checkout.ts", help = "Filename on the editor tab.")]
    pub title: String,
    #[setting(
        "Language",
        default = "tsx",
        help = "Language badge on the right of the header.",
        advanced
    )]
    pub language: String,
    #[setting(
        "Start line",
        default = 1,
        help = "First line number in the gutter, for excerpts.",
        advanced
    )]
    pub start_line: u32,
    #[setting("Line numbers", default = true, help = "Shows the gutter.")]
    pub show_line_numbers: bool,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Caret, focus band and glow color.",
        check = colour
    )]
    pub accent_color: String,
    #[setting("Theme", default = DarkOrLight::Dark, help = "Editor palette.")]
    pub theme: DarkOrLight,
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
        .fonts(&["JetBrains Mono"])
        .look(NodeLook::of("lucide:code")),
    )
}
