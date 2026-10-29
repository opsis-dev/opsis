use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct GlassCodeBlock {
    #[setting("Title", default = "hero.tsx")]
    pub title: String,
    #[setting("Width", default = 760, range(min = 300, max = 1100), step = 10)]
    pub width: u32,
    #[setting("Height", default = 460, range(min = 200, max = 700), step = 10)]
    pub height: u32,
    #[setting("Font size", default = 16, range(min = 10, max = 28), step = 1)]
    pub font_size: u32,
    #[setting("Glass color", default = "rgba(10, 10, 10, 0.6)", check = colour)]
    pub glass_color: String,
    #[setting(
        "Line stagger",
        default = 0.1333,
        range(min = 0, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub stagger_frames: f64,
    #[setting("Traffic lights", default = true)]
    pub show_traffic_lights: bool,
    #[setting(
        "Code",
        default = "export async function publish(draft: Draft) {\n  const page = await render(draft);\n  await upload(page, { public: true });\n  return page.url;\n}",
        widget = "code"
    )]
    pub code: String,
    #[setting("Glow behind the window")]
    pub aura: bool,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
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
