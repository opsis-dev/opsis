use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{
    LaptopEntrance, LaptopFinale, PhoneEntrance, PhoneVariant, RecordingEntrance, RecordingFit,
};
use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ScreenRecordingCamera {
    #[schemars(description = "At", range(min = 0, max = 120), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub at: f64,
    #[default = 0.8333]
    #[schemars(description = "Length", range(min = 0.0333, max = 20), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub duration: f64,
    #[default = 1.6]
    #[schemars(description = "Zoom", range(min = 1, max = 6), extend("x-step" = 0.1, "x-unit" = "×"))]
    pub zoom: f64,
    #[default = 0.5]
    #[schemars(description = "Across", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Down", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LaptopFrame {
    #[default = "app:placeholders/screen-wide.svg"]
    #[schemars(description = "Screen (image or video)", extend("x-widget" = "asset"))]
    pub screen_src: String,
    #[default(LaptopEntrance::Open)]
    #[schemars(description = "Entrance")]
    pub entrance: LaptopEntrance,
    #[default(LaptopFinale::ZoomToScreen)]
    #[schemars(description = "Finale")]
    pub finale: LaptopFinale,
    #[default = "Recording started"]
    #[schemars(description = "Notch label")]
    pub notch_label: String,
    #[default = true]
    #[schemars(description = "Notch")]
    pub show_notch: bool,
    #[default = 85.0]
    #[schemars(description = "Battery level", range(min = 0, max = 100), extend("x-step" = 1))]
    pub battery_level: f64,
    #[default = "#1D1D1F"]
    #[schemars(description = "Bezel color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub bezel_color: String,
    #[default = "#FFFFFF"]
    #[schemars(description = "Screen color", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub screen_color: String,
    #[default = 18.0]
    #[schemars(description = "Lid tilt", range(min = 0, max = 45), extend("x-step" = 1))]
    pub rest_tilt: f64,
    #[default = 22.0]
    #[schemars(description = "Lid radius", range(min = 8, max = 40), extend("x-step" = 1))]
    pub radius: f64,
    #[default = 0.85]
    #[schemars(description = "Scale", range(min = 0.4, max = 1.4), extend("x-step" = 0.05))]
    pub scale: f64,
    #[default = true]
    #[schemars(description = "Float loop")]
    pub float_loop: bool,
    #[default = 4.0]
    #[schemars(description = "Float amplitude", range(min = 0, max = 16), extend("x-step" = 1))]
    pub float_amplitude: f64,
    #[default = "0 40px 80px rgba(10,12,20,0.45)"]
    #[schemars(description = "Shadow")]
    pub shadow: String,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default = "#30D158"]
    #[schemars(description = "Recording light", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub indicator_color: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PhoneFrame {
    #[default(PhoneVariant::Showcase)]
    #[schemars(description = "Variant")]
    pub variant: PhoneVariant,
    #[default(PhoneEntrance::Float)]
    #[schemars(description = "Entrance")]
    pub entrance: PhoneEntrance,
    #[default = "app:placeholders/screen-tall.svg"]
    #[schemars(description = "Screen (image or video)", extend("x-widget" = "asset"))]
    pub screen_src: String,
    #[default = "#C7C9CE"]
    #[schemars(description = "Bezel color", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub bezel_color: String,
    #[default = "#0A0A0B"]
    #[schemars(description = "Screen color", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub screen_color: String,
    #[default = 53.0]
    #[schemars(description = "Body radius", range(min = 24, max = 90), extend("x-step" = 1))]
    pub radius: f64,
    #[default = 41.0]
    #[schemars(description = "Screen radius", range(min = 12, max = 78), extend("x-step" = 1))]
    pub screen_radius: f64,
    #[default = 0.92]
    #[schemars(description = "Scale", range(min = 0.4, max = 1.5), extend("x-step" = 0.05))]
    pub scale: f64,
    #[default(-12.0)]
    #[schemars(description = "Tilt angle", range(min = -30, max = 30), extend("x-step" = 1))]
    pub tilt_angle: f64,
    #[default = true]
    #[schemars(description = "Dynamic island")]
    pub show_dynamic_island: bool,
    #[schemars(description = "Float loop")]
    pub float_loop: bool,
    #[default = 6.0]
    #[schemars(description = "Float amplitude", range(min = 0, max = 20), extend("x-step" = 1))]
    pub float_amplitude: f64,
    #[default = "0 30px 70px rgba(16,24,40,0.28)"]
    #[schemars(description = "Shadow")]
    pub shadow: String,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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
    pub radius: f64,
    #[schemars(description = "Trim in", range(min = 0, max = 300), extend("x-step" = 1))]
    pub trim_before: f64,
    #[schemars(description = "Audio")]
    pub audio: bool,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default(vec![
        ScreenRecordingCamera { at: 1.1333, x: 0.36, y: 0.42, ..Default::default() },
        ScreenRecordingCamera { at: 3.2, zoom: 1.0, ..Default::default() },
    ])]
    #[schemars(description = "Camera moves", extend("x-help" = "Each move zooms to a spot. Leave the list empty for a still shot."))]
    pub camera: Vec<ScreenRecordingCamera>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<LaptopFrame>(
            element(
                ElementKind::Component,
                "snapcn/laptop-frame",
                "Laptop frame",
                "A laptop that opens, shows a notification in its notch, then moves the camera into the screen until your clip fills the frame.",
                C::Device,
                240,
            )
                .source("snapcn", "https://snapcn.dev/docs/laptop-frame", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:laptop").moves(NodeMotion::Open)),
        ),
        meta::<PhoneFrame>(
            element(
                ElementKind::Component,
                "snapcn/phone-frame",
                "Phone frame",
                "A phone frame with a notch and a screen slot for your clip, flat or tilted, with rise, rotate-in or float entrances.",
                C::Device,
                240,
            )
                .source("snapcn", "https://snapcn.dev/docs/phone-frame", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:smartphone")),
        ),
        meta::<ScreenRecording>(
            element(
                ElementKind::Component,
                "snapcn/screen-recording",
                "Screen recording",
                "Screen recording zoom: crop and auto-zoom a product demo capture. Takes a video or a still through the same track.",
                C::Device,
                240,
            )
                .source("snapcn", "https://snapcn.dev/docs/screen-recording", "MIT")
                .look(NodeLook::of("lucide:monitor-play")),
        ),
    ]
}
