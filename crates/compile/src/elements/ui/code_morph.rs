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
pub struct CodeMorphStep {
    #[setting("Code", widget = "code")]
    pub code: String,
    #[setting(
        "Show at",
        range(min = 0, max = 120),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub at: f64,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CodeMorph {
    #[setting("Filename", default = "invoices.tsx")]
    pub filename: String,
    #[setting("Theme", default = DarkOrLight::Dark)]
    pub theme: DarkOrLight,
    #[setting("Accent", default = "#0ea5e9", check = colour)]
    pub accent_color: String,
    #[setting(
        "Morph time",
        default = 1.2,
        range(min = 0.4, max = 3),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub morph_duration: f64,
    #[setting("Font size", default = 20, range(min = 12, max = 32), step = 1)]
    pub font_size: u32,
    #[setting("Highlight changes", default = true)]
    pub highlight: bool,
    #[setting("Line numbers", default = true)]
    pub line_numbers: bool,
    #[setting("Window dots", default = true)]
    pub window_dots: bool,
    #[setting(
        "Versions",
        default = vec![
            CodeMorphStep {
                code: "import { loadInvoices } from \"./api\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  const invoices = await loadInvoices();\n  list.textContent = \"\";\n  for (const invoice of invoices) {\n    const row = document.createElement(\"li\");\n    row.textContent = invoice.title;\n    list.append(row);\n  }\n}".into(),
                ..Default::default()
            },
            CodeMorphStep {
                at: 1.6,
                code: "import { loadInvoices } from \"./api\";\nimport { renderRows } from \"./rows\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  renderRows(list, await loadInvoices());\n}".into(),
            },
        ],
        help = "Each version of the code and when it starts to show."
    )]
    pub steps: Vec<CodeMorphStep>,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<CodeMorph>(
            element(
                ElementKind::Component,
                "code-morph",
                "Code morph",
                "A flat code editor that morphs between versions of a file, gliding each surviving token to its new line and column while removed code dissolves and new code fades in.",
                ElementCategory::Ui,
                138,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:code-xml").morphs("lucide:braces")),
        )
}
