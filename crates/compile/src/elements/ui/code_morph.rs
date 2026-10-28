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
pub struct CodeMorphStep {
    #[schemars(description = "Code", extend("x-widget" = "code"))]
    pub code: String,
    #[schemars(
        description = "Show at",
        range(min = 0, max = 120),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub at: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CodeMorph {
    #[default = "invoices.tsx"]
    #[schemars(description = "Filename")]
    pub filename: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme")]
    pub theme: DarkOrLight,
    #[default = "#0ea5e9"]
    #[schemars(
        description = "Accent",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent_color: String,
    #[default = 1.2]
    #[schemars(
        description = "Morph time",
        range(min = 0.4, max = 3),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub morph_duration: f64,
    #[default = 20]
    #[schemars(description = "Font size", range(min = 12, max = 32), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = true]
    #[schemars(description = "Highlight changes")]
    pub highlight: bool,
    #[default = true]
    #[schemars(description = "Line numbers")]
    pub line_numbers: bool,
    #[default = true]
    #[schemars(description = "Window dots")]
    pub window_dots: bool,
    #[default(vec![
        CodeMorphStep {
            code: "import { loadInvoices } from \"./api\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  const invoices = await loadInvoices();\n  list.textContent = \"\";\n  for (const invoice of invoices) {\n    const row = document.createElement(\"li\");\n    row.textContent = invoice.title;\n    list.append(row);\n  }\n}".into(),
            ..Default::default()
        },
        CodeMorphStep {
            at: 1.6,
            code: "import { loadInvoices } from \"./api\";\nimport { renderRows } from \"./rows\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  renderRows(list, await loadInvoices());\n}".into(),
        },
    ])]
    #[schemars(
        description = "Versions",
        extend("x-help" = "Each version of the code and when it starts to show.")
    )]
    pub steps: Vec<CodeMorphStep>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
