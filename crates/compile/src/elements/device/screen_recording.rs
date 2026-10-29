use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum RecordingFit {
    Cover,
    Contain,
}

#[choice(export)]
pub enum RecordingEntrance {
    Fade,
    None,
}

#[settings(export)]
pub struct ScreenRecordingCamera {
    #[setting(
        "At",
        range(min = 0, max = 120),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub at: f64,
    #[setting(
        "Length",
        default = 0.8333,
        range(min = 0.0333, max = 20),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub duration: f64,
    #[setting("Zoom", default = 1.6, range(min = 1, max = 6), step = 0.1, unit = "×")]
    pub zoom: f64,
    #[setting("Across", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Down", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
}

#[settings(export)]
pub struct ScreenRecording {
    #[setting(
        "Recording",
        default = "app:placeholders/screen-wide.svg",
        widget = "asset"
    )]
    pub src: String,
    #[setting("Crop top", range(min = 0, max = 0.4), step = 0.005, advanced)]
    pub crop_top: f64,
    #[setting("Crop right", range(min = 0, max = 0.4), step = 0.005)]
    pub crop_right: f64,
    #[setting("Crop bottom", range(min = 0, max = 0.4), step = 0.005)]
    pub crop_bottom: f64,
    #[setting("Crop left", range(min = 0, max = 0.4), step = 0.005)]
    pub crop_left: f64,
    #[setting("Fit", default = RecordingFit::Cover)]
    pub fit: RecordingFit,
    #[setting("Letterbox", widget = "color", zero_label = "Theme", advanced)]
    pub backdrop_color: String,
    #[setting("Entrance", default = RecordingEntrance::Fade)]
    pub entrance: RecordingEntrance,
    #[setting("Corner radius", range(min = 0, max = 48), step = 1, advanced)]
    pub radius: u32,
    #[setting("Trim in", range(min = 0, max = 300), step = 1, advanced)]
    pub trim_before: u32,
    #[setting("Audio")]
    pub audio: bool,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme",
        advanced
    )]
    pub theme: Theme,
    #[setting(
        "Camera moves",
        default = vec![
            ScreenRecordingCamera { at: 1.1333, x: 0.36, y: 0.42, ..Default::default() },
            ScreenRecordingCamera { at: 3.2, zoom: 1.0, ..Default::default() },
        ],
        help = "Each move zooms to a spot. Leave the list empty for a still shot."
    )]
    pub camera: Vec<ScreenRecordingCamera>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Source shape",
        range(min = 0.2, max = 5),
        step = 0.01,
        zero_label = "Frame's shape",
        help = "The recording's width divided by its height.",
        advanced
    )]
    pub source_aspect: Option<f64>,
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
