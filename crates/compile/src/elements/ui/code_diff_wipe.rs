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
pub struct CodeDiffWipe {
    #[setting(
        "Before",
        default = "export function render(scene) {\n  const frames = scene.frames;\n  return frames.map(draw);\n}",
        widget = "code",
        help = "Source before the patch."
    )]
    pub before: String,
    #[setting(
        "After",
        default = "export function render(scene, options) {\n  const frames = scene.frames;\n  const scale = options.scale ?? 1;\n  return frames.map((frame) => draw(frame, scale));\n}",
        widget = "code",
        help = "Source after the patch; the scene diffs the two."
    )]
    pub after: String,
    #[setting(
        "Title",
        default = "render.ts",
        help = "Filename on the window header."
    )]
    pub title: String,
    #[setting(
        "Wipe (seconds)",
        default = 1.7,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "Seconds the apply front takes to travel the file.",
        advanced
    )]
    pub wipe_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Apply front and glow color.",
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
