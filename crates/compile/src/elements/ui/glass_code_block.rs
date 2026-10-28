use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct GlassCodeBlock {
    #[default = "hero.tsx"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = 760]
    #[schemars(description = "Width", range(min = 300, max = 1100), extend("x-step" = 10))]
    pub width: u32,
    #[default = 460]
    #[schemars(description = "Height", range(min = 200, max = 700), extend("x-step" = 10))]
    pub height: u32,
    #[default = 16]
    #[schemars(description = "Font size", range(min = 10, max = 28), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = "rgba(10, 10, 10, 0.6)"]
    #[schemars(
        description = "Glass color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub glass_color: String,
    #[default = 0.1333]
    #[schemars(
        description = "Line stagger",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub stagger_frames: f64,
    #[default = true]
    #[schemars(description = "Traffic lights")]
    pub show_traffic_lights: bool,
    #[default = "export async function publish(draft: Draft) {\n  const page = await render(draft);\n  await upload(page, { public: true });\n  return page.url;\n}"]
    #[schemars(description = "Code", extend("x-widget" = "code"))]
    pub code: String,
    #[schemars(description = "Glow behind the window")]
    pub aura: bool,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GlassCodeBlock>(
        element(
            ElementKind::Component,
            "glass-code-block",
            "Glass code block",
            "A frosted-glass code editor that reveals the code line by line.",
            ElementCategory::Ui,
            180,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:square-code")),
    )
}
