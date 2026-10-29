use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, DarkOrLight};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
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
        help = "Colours for branch lanes 1 and up."
    )]
    pub lane_colors: Vec<Colour>,
    #[setting("Theme", default = DarkOrLight::Dark, help = "Surface palette.")]
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
