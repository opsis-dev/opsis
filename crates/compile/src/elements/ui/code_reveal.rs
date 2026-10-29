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
    #[setting(
        "Font size",
        default = 21,
        range(min = 12, max = 32),
        step = 1,
        unit = "px"
    )]
    pub font_size: u32,
    #[setting(
        "Window width",
        default = 940,
        range(min = 400, max = 1200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub window_width: u32,
    #[setting("Line height", default = 1.62, range(min = 1.2, max = 2.4), advanced)]
    pub line_height: f64,
    #[setting(
        "Visible lines",
        default = 10,
        range(min = 3, max = 30),
        step = 1,
        advanced
    )]
    pub visible_lines: u32,
    #[setting(
        "Typing speed",
        default = 55.0,
        range(min = 5, max = 200),
        unit = "characters a second",
        advanced
    )]
    pub typing_speed: f64,
    #[setting(
        "Pause at line ends",
        default = 0.07,
        range(min = 0, max = 1),
        unit = "s",
        advanced
    )]
    pub line_pause: f64,
    #[setting(
        "Start typing at",
        default = 0.4,
        range(min = 0, max = 10),
        unit = "s",
        advanced
    )]
    pub start_at: f64,
    #[setting(
        "Highlight after",
        default = 0.34,
        range(min = 0, max = 5),
        unit = "s",
        advanced
    )]
    pub highlight_after: f64,
    #[setting("Other lines", default = 0.34, range(min = 0, max = 1), advanced)]
    pub dim_others: f64,
    #[setting("Glow behind the window", default = true, advanced)]
    pub glow: bool,
    #[setting("Vignette", default = true, advanced)]
    pub vignette: bool,
    #[setting(
        "Corner radius",
        default = 16,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Window dots", default = true, advanced)]
    pub window_dots: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
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
