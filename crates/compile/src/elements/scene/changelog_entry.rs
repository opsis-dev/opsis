use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeKind {
    Added,
    Fixed,
    Changed,
    Removed,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ChangelogEntryChange {
    #[setting("Kind", default = ChangeKind::Removed)]
    pub kind: ChangeKind,
    #[setting("Text", default = "Legacy waveform atom")]
    pub text: String,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ChangelogEntry {
    #[setting(
        "Changes",
        default = vec![
            ChangelogEntryChange {
                kind: ChangeKind::Added,
                text: "Caption presets for nine languages".into(),
            },
            ChangelogEntryChange {
                kind: ChangeKind::Added,
                text: "Render API webhooks on job completion".into(),
            },
            ChangelogEntryChange {
                kind: ChangeKind::Fixed,
                text: "Retry backoff no longer stalls the queue".into(),
            },
            ChangelogEntryChange {
                kind: ChangeKind::Changed,
                text: "Theme tokens moved to a single source".into(),
            },
            ChangelogEntryChange::default(),
        ],
        help = "A kind (added, fixed, changed or removed) and the sentence beside it."
    )]
    pub changes: Vec<ChangelogEntryChange>,
    #[setting(
        "Version",
        default = "v2.4.0",
        help = "Version string, set at display size."
    )]
    pub version: String,
    #[setting("Date", default = "16 August", help = "Date beside the version.")]
    pub date: String,
    #[setting(
        "Summary",
        default = "Captions, webhooks, and a queue that stops sulking.",
        help = "Headline under the version. Omit for releases without one."
    )]
    pub summary: String,
    #[setting(
        "Start at (seconds)",
        default = 0.66,
        step = 0.01,
        help = "Second the first change row arrives.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Stagger (seconds)",
        default = 0.28,
        step = 0.01,
        help = "Seconds between rows. Each sentence trails its own tag by 0.08s.",
        advanced
    )]
    pub stagger_seconds: f64,
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
    #[setting("Added colour", widget = "color", zero_label = "Default")]
    pub kind_added: String,
    #[setting("Fixed colour", widget = "color", zero_label = "Default")]
    pub kind_fixed: String,
    #[setting("Changed colour", widget = "color", zero_label = "Default")]
    pub kind_changed: String,
    #[setting("Removed colour", widget = "color", zero_label = "Default")]
    pub kind_removed: String,
}

pub fn definition() -> ElementMeta {
    meta::<ChangelogEntry>(
        element(
            ElementKind::Component,
            "changelog-entry",
            "Changelog",
            "A release note: version and date, then each change appears with its own tag.",
            ElementCategory::Scene,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:rotate-ccw-clock")),
    )
}
