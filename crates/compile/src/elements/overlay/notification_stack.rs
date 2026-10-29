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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationStackToast {
    #[setting("Title", default = "Upload failed")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Body", "type" = "string")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Tone", "type" = "string", "enum" = ["info", "success", "warn", "error"])]
    pub tone: Option<ToastTone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Meta", "type" = "string")]
    pub meta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("At seconds", "type" = "number")]
    pub at_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Hold for", "type" = "number")]
    pub hold_for: Option<f64>,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationStack {
    #[setting(
        "Notifications",
        default = vec![
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
        ],
        help = "Title, optional body and meta, tone, and optional per-toast arrival and dwell."
    )]
    pub toasts: Vec<NotificationStackToast>,
    #[setting(
        "Position",
        default = ToastCorner::TopRight,
        help = "Corner the stack anchors to. Bottom anchors grow upward."
    )]
    pub align: ToastCorner,
    #[setting(
        "Start at (seconds)",
        default = 0.3,
        step = 0.01,
        help = "Second the first toast arrives.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting("Stagger (seconds)", default = 0.62, step = 0.01, advanced)]
    pub stagger_seconds: f64,
    #[setting(
        "Time on screen (seconds)",
        default = 2.3,
        step = 0.1,
        help = "Default seconds a toast stays up before dismissing itself.",
        advanced
    )]
    pub life_seconds: f64,
    #[setting(
        "Show timer bar",
        default = true,
        help = "Draining line along each toast's bottom edge."
    )]
    pub show_progress: bool,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Colour used by the warn tone.",
        check = colour
    )]
    pub accent_color: String,
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
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:bell")),
    )
}
