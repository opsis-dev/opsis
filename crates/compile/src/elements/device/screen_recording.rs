use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum RecordingFit {
    Cover,
    Contain,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum RecordingEntrance {
    Fade,
    None,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ScreenRecordingCamera {
    #[schemars(
        description = "At",
        range(min = 0, max = 120),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub at: f64,
    #[default = 0.8333]
    #[schemars(
        description = "Length",
        range(min = 0.0333, max = 20),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub duration: f64,
    #[default = 1.6]
    #[schemars(
        description = "Zoom",
        range(min = 1, max = 6),
        extend("x-step" = 0.1, "x-unit" = "×")
    )]
    pub zoom: f64,
    #[default = 0.5]
    #[schemars(description = "Across", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Down", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub y: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ScreenRecording {
    #[default = "app:placeholders/screen-wide.svg"]
    #[schemars(description = "Recording", extend("x-widget" = "asset"))]
    pub src: String,
    #[schemars(description = "Crop top", range(min = 0, max = 0.4), extend("x-step" = 0.005))]
    pub crop_top: f64,
    #[schemars(description = "Crop right", range(min = 0, max = 0.4), extend("x-step" = 0.005))]
    pub crop_right: f64,
    #[schemars(description = "Crop bottom", range(min = 0, max = 0.4), extend("x-step" = 0.005))]
    pub crop_bottom: f64,
    #[schemars(description = "Crop left", range(min = 0, max = 0.4), extend("x-step" = 0.005))]
    pub crop_left: f64,
    #[default(RecordingFit::Cover)]
    #[schemars(description = "Fit")]
    pub fit: RecordingFit,
    #[schemars(description = "Letterbox", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub backdrop_color: String,
    #[default(RecordingEntrance::Fade)]
    #[schemars(description = "Entrance")]
    pub entrance: RecordingEntrance,
    #[schemars(description = "Corner radius", range(min = 0, max = 48), extend("x-step" = 1))]
    pub radius: u32,
    #[schemars(description = "Trim in", range(min = 0, max = 300), extend("x-step" = 1))]
    pub trim_before: u32,
    #[schemars(description = "Audio")]
    pub audio: bool,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default(vec![
        ScreenRecordingCamera { at: 1.1333, x: 0.36, y: 0.42, ..Default::default() },
        ScreenRecordingCamera { at: 3.2, zoom: 1.0, ..Default::default() },
    ])]
    #[schemars(
        description = "Camera moves",
        extend("x-help" = "Each move zooms to a spot. Leave the list empty for a still shot.")
    )]
    pub camera: Vec<ScreenRecordingCamera>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ScreenRecording>(
            element(
                ElementKind::Component,
                "screen-recording",
                "Screen recording",
                "Screen recording zoom: crop and auto-zoom a product demo capture. Takes a video or a still through the same track.",
                ElementCategory::Device,
                240,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:monitor-play")),
        )
}
