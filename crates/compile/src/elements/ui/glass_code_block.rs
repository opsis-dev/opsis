use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
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
        frames_at = 30,
        advanced
    )]
    pub stagger_frames: f64,
    #[setting("Traffic lights", default = true, advanced)]
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
    #[setting(
        "Frost",
        default = 16,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub frost: u32,
    #[setting(
        "Corner radius",
        default = 16,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Line height", default = 1.55, range(min = 1.2, max = 2.4), advanced)]
    pub line_height: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
    #[setting("Code colour", default = "#e4e4e7", widget = "color", check = colour, advanced)]
    pub code_color: String,
    #[setting("Comment colour", default = "#52525b", widget = "color", check = colour, advanced)]
    pub comment_color: String,
    #[setting("Text in quotes colour", default = "#86efac", widget = "color", check = colour, advanced)]
    pub string_color: String,
    #[setting("Keyword colour", default = "#c4b5fd", widget = "color", check = colour, advanced)]
    pub keyword_color: String,
    #[setting("Number colour", default = "#fcd34d", widget = "color", check = colour, advanced)]
    pub number_color: String,
    #[setting("Title colour", default = "#a1a1aa", widget = "color", check = colour, advanced)]
    pub title_color: String,
    #[setting("Line number colour", default = "#3f3f46", widget = "color", check = colour, advanced)]
    pub line_number_color: String,
    #[setting(
        "Glow colours",
        default = vec!["rgba(56,189,248,0.22)".into(), "rgba(168,85,247,0.18)".into()],
        length(min = 2, max = 2),
        advanced
    )]
    pub aura_colors: Vec<Colour>,
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
