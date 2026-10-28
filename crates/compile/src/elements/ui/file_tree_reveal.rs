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
pub struct FileTreeReveal {
    #[default(vec![
        "src/components/cart.tsx".into(),
        "src/components/checkout.tsx".into(),
        "src/lib/api.ts".into(),
        "src/index.ts".into(),
        "public/logo.svg".into(),
        "README.md".into(),
        "package.json".into(),
    ])]
    #[schemars(
        description = "Files",
        extend(
            "x-help" = "One path per item, like src/app.ts. End a folder with / to show it empty."
        )
    )]
    pub nodes: Vec<String>,
    #[default = "acme-app"]
    #[schemars(
        description = "Title",
        extend("x-help" = "Title bar over the tree. Omit to drop the header.")
    )]
    pub title: String,
    #[default = "src/components/checkout.tsx"]
    #[schemars(
        description = "Highlighted file",
        extend(
            "x-help" = "Slash-joined path of the file that lights up at the end. Omit to select nothing."
        )
    )]
    pub selected_path: String,
    #[default = 0.18]
    #[schemars(
        description = "Row stagger",
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between one row appearing and the next.",
            "x-advanced" = true
        )
    )]
    pub row_stagger: f64,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Selected row's bar, tint and label.",
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
    meta::<FileTreeReveal>(
        element(
            ElementKind::Component,
            "file-tree-reveal",
            "File tree",
            "A folder tree that opens one item at a time and highlights a file.",
            ElementCategory::Ui,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:folder-tree")),
    )
}
