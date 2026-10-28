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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ChangelogEntryChange {
    #[default(ChangeKind::Removed)]
    #[schemars(description = "Kind")]
    pub kind: ChangeKind,
    #[default = "Legacy waveform atom"]
    #[schemars(description = "Text")]
    pub text: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ChangelogEntry {
    #[default(vec![
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
    ])]
    #[schemars(
        description = "Changes",
        extend("x-help" = "A kind (added, fixed, changed or removed) and the sentence beside it.")
    )]
    pub changes: Vec<ChangelogEntryChange>,
    #[default = "v2.4.0"]
    #[schemars(description = "Version", extend("x-help" = "Version string, set at display size."))]
    pub version: String,
    #[default = "16 August"]
    #[schemars(description = "Date", extend("x-help" = "Date beside the version."))]
    pub date: String,
    #[default = "Captions, webhooks, and a queue that stops sulking."]
    #[schemars(
        description = "Summary",
        extend("x-help" = "Headline under the version. Omit for releases without one.")
    )]
    pub summary: String,
    #[default = 0.66]
    #[schemars(
        description = "Start at (seconds)",
        extend(
            "x-step" = 0.01,
            "x-help" = "Second the first change row arrives.",
            "x-advanced" = true
        )
    )]
    pub start_at_seconds: f64,
    #[default = 0.28]
    #[schemars(
        description = "Stagger (seconds)",
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between rows. Each sentence trails its own tag by 0.08s.",
            "x-advanced" = true
        )
    )]
    pub stagger_seconds: f64,
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
    #[schemars(
        description = "Added colour",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub kind_added: String,
    #[schemars(
        description = "Fixed colour",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub kind_fixed: String,
    #[schemars(
        description = "Changed colour",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub kind_changed: String,
    #[schemars(
        description = "Removed colour",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
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
