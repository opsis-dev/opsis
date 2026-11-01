use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct CommitGraphCommit {
    #[setting("Message", default = "Merge retry backoff")]
    pub message: String,
    #[setting("Hash", default = "b8c4e05")]
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Lane", "type" = "integer")]
    pub lane: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Parent", "type" = "integer")]
    pub parent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Merge from", "type" = "integer")]
    pub merge_from: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Ref", "type" = "string")]
    pub r#ref: Option<String>,
}

#[settings(export)]
pub struct CommitGraph {
    #[setting(
        "Commits",
        default = vec![
            CommitGraphCommit {
                message: "Seed the registry manifest".into(),
                hash: "9c41f0a".into(),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Add render queue worker".into(),
                hash: "3ab77e2".into(),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Branch: retry backoff".into(),
                hash: "d0e91c4".into(),
                lane: Some(1),
                parent: Some(1),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Cap retries at five".into(),
                hash: "51b2fa8".into(),
                lane: Some(1),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Ship caption presets".into(),
                hash: "7f30dd1".into(),
                lane: Some(0),
                parent: Some(1),
                ..Default::default()
            },
            CommitGraphCommit {
                lane: Some(0),
                parent: Some(4),
                merge_from: Some(3),
                r#ref: Some("main".into()),
                ..Default::default()
            },
        ]
    )]
    pub commits: Vec<CommitGraphCommit>,
    #[setting(
        "Window title",
        default = "git log --graph --oneline",
        help = "Text in the title bar. Omit to drop the chrome header."
    )]
    pub window_title: String,
    #[setting(
        "Start at (seconds)",
        default = 0.34,
        step = 0.01,
        help = "Second the first commit lands.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Step (seconds)",
        default = 0.36,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "Seconds between commits. Each edge draws over this same gap.",
        advanced
    )]
    pub step_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Trunk colour, and the branch ref chip.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Branch colors",
        default = vec!["#7DD3E8".into(), "#C99BE8".into(), "#9BD4A0".into(), "#E8B86D".into()],
        length(min = 1),
        help = "Colours for branch lanes 1 and up.",
        advanced
    )]
    pub lane_colors: Vec<Colour>,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = code_themes,
        widget = "theme"
    )]
    pub theme: Theme,
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
    #[setting("Hash font", widget = "font", advanced)]
    pub code_font: String,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
    #[setting(
        "Window width",
        default = 860,
        range(min = 400, max = 1400),
        step = 1,
        unit = "px",
        advanced
    )]
    pub window_width: u32,
    #[setting(
        "Row spacing",
        default = 48,
        range(min = 28, max = 96),
        step = 1,
        unit = "px",
        advanced
    )]
    pub row_spacing: u32,
    #[setting(
        "Lane spacing",
        default = 30,
        range(min = 16, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub lane_spacing: u32,
    #[setting(
        "Dot size",
        default = 6.5,
        range(min = 3, max = 16),
        unit = "px",
        advanced
    )]
    pub dot_size: f64,
    #[setting(
        "Line width",
        default = 2.2,
        range(min = 1, max = 8),
        unit = "px",
        advanced
    )]
    pub line_width: f64,
    #[setting(
        "Text size",
        default = 15.5,
        range(min = 10, max = 28),
        unit = "px",
        advanced
    )]
    pub text_size: f64,
    #[setting(
        "Corner radius",
        default = 18,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Window dots", default = true, advanced)]
    pub window_dots: bool,
}

pub fn definition() -> ElementMeta {
    meta::<CommitGraph>(
        element(
            ElementKind::Component,
            "commit-graph",
            "Commit graph",
            "A version history graph that draws its branches and merges commit by commit.",
            ElementCategory::Ui,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:git-graph").moves(NodeMotion::Draw)),
    )
}
