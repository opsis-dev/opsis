use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, steps_timeline_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct TimelineStep {
    #[setting("Title", default = "Next step")]
    pub title: String,
    #[setting("Description", default = "Close with one clear call to action.")]
    pub description: String,
}

#[settings(export)]
pub struct StepsTimeline {
    #[setting("Title", default = "How we read the quarter")]
    pub title: String,
    #[setting("Small heading")]
    pub eyebrow: String,
    #[setting(
        "Steps",
        default = vec![
            TimelineStep {
                title: "Hook".into(),
                description: "Lead with the sharpest insight.".into(),
            },
            TimelineStep {
                title: "Proof".into(),
                description: "Show the chart and the numbers.".into(),
            },
            TimelineStep::default(),
        ],
        length(min = 1, max = 5),
        help = "Up to five steps, checked off one after another."
    )]
    pub steps: Vec<TimelineStep>,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = steps_timeline_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Accent color", default = "#f59e0b", check = colour)]
    pub accent_color: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Set it to transparent to lay the timeline over other layers."
    )]
    pub background_color: String,
    #[setting("Speed", default = 1.5, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Text size",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        advanced
    )]
    pub text_size: f64,
    #[setting("Uppercase small heading", default = true, advanced)]
    pub eyebrow_caps: bool,
    #[setting("Glow", default = 1.0, range(min = 0, max = 3), step = 0.05, advanced)]
    pub glow: f64,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<StepsTimeline>(
        element(
            ElementKind::Component,
            "steps-timeline",
            "Steps timeline",
            "A row of steps that check off one after another along a line, each with its title and a short description.",
            ElementCategory::Data,
            150,
        )
        .look(NodeLook::of("lucide:route").moves(opsis_types::looks::NodeMotion::Draw)),
    )
}
