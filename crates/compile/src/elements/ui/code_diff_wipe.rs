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
pub struct CodeDiffWipe {
    #[default = "export function render(scene) {\n  const frames = scene.frames;\n  return frames.map(draw);\n}"]
    #[schemars(
        description = "Before",
        extend("x-widget" = "code", "x-help" = "Source before the patch.")
    )]
    pub before: String,
    #[default = "export function render(scene, options) {\n  const frames = scene.frames;\n  const scale = options.scale ?? 1;\n  return frames.map((frame) => draw(frame, scale));\n}"]
    #[schemars(
        description = "After",
        extend("x-widget" = "code", "x-help" = "Source after the patch; the scene diffs the two.")
    )]
    pub after: String,
    #[default = "render.ts"]
    #[schemars(description = "Title", extend("x-help" = "Filename on the window header."))]
    pub title: String,
    #[default = 1.7]
    #[schemars(
        description = "Wipe (seconds)",
        range(min = 0.05, max = 10),
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds the apply front takes to travel the file.",
            "x-advanced" = true
        )
    )]
    pub wipe_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Apply front and glow color.",
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
    meta::<CodeDiffWipe>(
            element(
                ElementKind::Component,
                "code-diff-wipe",
                "Code diff",
                "A code editor that applies a change line by line: removed lines collapse and new lines type in.",
                ElementCategory::Ui,
                135,
            )
                .fonts(&["JetBrains Mono"])
                .look(NodeLook::of("lucide:file-diff")),
        )
}
