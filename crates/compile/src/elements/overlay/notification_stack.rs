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
pub enum ToastTone {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum ToastCorner {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationStackToast {
    #[default = "Upload failed"]
    #[schemars(description = "Title")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Body", extend("type" = "string"))]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Tone",
        extend("type" = "string", "enum" = ["info", "success", "warn", "error"])
    )]
    pub tone: Option<ToastTone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Meta", extend("type" = "string"))]
    pub meta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "At seconds", extend("type" = "number"))]
    pub at_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Hold for", extend("type" = "number"))]
    pub hold_for: Option<f64>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationStack {
    #[default(vec![
        NotificationStackToast {
            title: "Render finished".into(),
            body: Some("launch-teaser.mp4 · 1080p".into()),
            tone: Some(ToastTone::Success),
            meta: Some("now".into()),
            ..Default::default()
        },
        NotificationStackToast {
            title: "3 new comments".into(),
            body: Some("Mina left notes on scene 4".into()),
            tone: Some(ToastTone::Info),
            meta: Some("1m".into()),
            ..Default::default()
        },
        NotificationStackToast {
            title: "Storage at 86%".into(),
            body: Some("Archive old renders to free space".into()),
            tone: Some(ToastTone::Warn),
            meta: Some("2m".into()),
            ..Default::default()
        },
        NotificationStackToast {
            body: Some("b-roll-rooftop.mov · retrying".into()),
            tone: Some(ToastTone::Error),
            meta: Some("2m".into()),
            ..Default::default()
        },
    ])]
    #[schemars(
        description = "Notifications",
        extend(
            "x-help" = "Title, optional body and meta, tone, and optional per-toast arrival and dwell."
        )
    )]
    pub toasts: Vec<NotificationStackToast>,
    #[default(ToastCorner::TopRight)]
    #[schemars(
        description = "Position",
        extend("x-help" = "Corner the stack anchors to. Bottom anchors grow upward.")
    )]
    pub align: ToastCorner,
    #[default = 0.3]
    #[schemars(
        description = "Start at (seconds)",
        extend("x-step" = 0.01, "x-help" = "Second the first toast arrives.", "x-advanced" = true)
    )]
    pub start_at_seconds: f64,
    #[default = 0.62]
    #[schemars(description = "Stagger (seconds)", extend("x-step" = 0.01, "x-advanced" = true))]
    pub stagger_seconds: f64,
    #[default = 2.3]
    #[schemars(
        description = "Time on screen (seconds)",
        extend(
            "x-step" = 0.1,
            "x-help" = "Default seconds a toast stays up before dismissing itself.",
            "x-advanced" = true
        )
    )]
    pub life_seconds: f64,
    #[default = true]
    #[schemars(
        description = "Show timer bar",
        extend("x-help" = "Draining line along each toast's bottom edge.")
    )]
    pub show_progress: bool,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Colour used by the warn tone.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
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
}

pub fn definition() -> ElementMeta {
    meta::<NotificationStack>(
        element(
            ElementKind::Component,
            "notification-stack",
            "Notification stack",
            "Notifications slide in, stack up and dismiss themselves.",
            ElementCategory::Overlay,
            120,
        )
        .look(NodeLook::of("lucide:bell")),
    )
}
