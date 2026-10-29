use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Corner;
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

#[settings(export)]
pub struct AiPromptFlow {
    #[setting("Theme", transform = neutral_themes, widget = "theme")]
    pub theme: Theme,
    #[setting("Prompt", default = "Summarize this thread")]
    pub prompt: String,
    #[setting("Button label", default = "Generate")]
    pub button_label: String,
    #[setting(
        "Answer",
        default = vec![
            "The thread debates the Q3 roadmap: ship the editor first,".into(),
            "defer billing to Q4, and pull the migration forward so".into(),
            "infra is unblocked before the team scales next quarter.".into(),
        ],
        help = "One line of the answer per item."
    )]
    pub answer_lines: Vec<String>,
    #[setting("Notice", default = "Response ready")]
    pub toast_title: String,
    #[setting(
        "Typing time",
        default = 1.67,
        range(min = 0.2, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub typing_seconds: f64,
    #[setting(
        "Thinking time",
        default = 2.87,
        range(min = 0.3, max = 8),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub loading_seconds: f64,
    #[setting(
        "Notice shown for",
        default = 2.0,
        range(min = 0.5, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub notice_seconds: f64,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4))]
    pub speed: f64,
    #[setting(
        "Prompt field width",
        default = 320,
        range(min = 200, max = 900),
        step = 1,
        unit = "px",
        advanced
    )]
    pub field_width: u32,
    #[setting(
        "Answer width",
        default = 560,
        range(min = 280, max = 1100),
        step = 1,
        unit = "px",
        advanced
    )]
    pub answer_width: u32,
    #[setting(
        "Answer text size",
        default = 18,
        range(min = 12, max = 36),
        step = 1,
        unit = "px",
        advanced
    )]
    pub answer_size: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting("Button style", default = ButtonVariant::Default, advanced)]
    pub button_variant: ButtonVariant,
    #[setting("Notice position", default = Corner::BottomRight, advanced)]
    pub notice_corner: Corner,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background_color: String,
    #[setting(
        "Corner radius",
        default = 10,
        range(min = 0, max = 32),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: u32,
}

pub fn definition() -> ElementMeta {
    meta::<AiPromptFlow>(
            element(
                ElementKind::Component,
                "ai-prompt-flow",
                "AI prompt flow",
                "An AI prompt composition: a prompt types into the field, the Generate button runs hover → press → loading, a skeleton shimmer reveals, then crossfades into the generated answer, and a ready toast slides in. A pure orchestrator — the shimmer is owned by the skeleton-block motion atom; every other channel comes from a composed primitive's hook.",
                ElementCategory::Scene,
                240,
            )
                .authored_clock()
                .exact_design(1280, 720)
                .look(NodeLook::of("lucide:wand-sparkles")),
        )
}
