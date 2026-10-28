use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, DarkOrLight};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitGraphCommit {
    #[default = "Merge retry backoff"]
    #[schemars(description = "Message")]
    pub message: String,
    #[default = "b8c4e05"]
    #[schemars(description = "Hash")]
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Lane", extend("type" = "integer"))]
    pub lane: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Parent", extend("type" = "integer"))]
    pub parent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Merge from", extend("type" = "integer"))]
    pub merge_from: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Ref", extend("type" = "string"))]
    pub r#ref: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitGraph {
    #[default(vec![
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
    ])]
    #[schemars(description = "Commits")]
    pub commits: Vec<CommitGraphCommit>,
    #[default = "git log --graph --oneline"]
    #[schemars(
        description = "Window title",
        extend("x-help" = "Text in the title bar. Omit to drop the chrome header.")
    )]
    pub window_title: String,
    #[default = 0.34]
    #[schemars(
        description = "Start at (seconds)",
        extend("x-step" = 0.01, "x-help" = "Second the first commit lands.", "x-advanced" = true)
    )]
    pub start_at_seconds: f64,
    #[default = 0.36]
    #[schemars(
        description = "Step (seconds)",
        range(min = 0.05, max = 10),
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between commits. Each edge draws over this same gap.",
            "x-advanced" = true
        )
    )]
    pub step_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Trunk colour, and the branch ref chip.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default(vec!["#7DD3E8".into(), "#C99BE8".into(), "#9BD4A0".into(), "#E8B86D".into()])]
    #[schemars(
        description = "Branch colors",
        length(min = 1),
        extend("x-help" = "Colours for branch lanes 1 and up.")
    )]
    pub lane_colors: Vec<Colour>,
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
    meta::<CommitGraph>(
        element(
            ElementKind::Component,
            "commit-graph",
            "Commit graph",
            "A version history graph that draws its branches and merges commit by commit.",
            ElementCategory::Ui,
            120,
        )
        .look(NodeLook::of("lucide:git-graph").moves(NodeMotion::Draw)),
    )
}
