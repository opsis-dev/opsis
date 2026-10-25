use serde::{Deserialize, Serialize};
use specta::Type;

use crate::actions::{Action, ActionLog};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum CameraStyle {
    Classic,
    Proof,
    Ledger,
    Stage,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum WindowStyle {
    Macos,
    Windows,
    Linux,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum FrameTheme {
    Light,
    Dark,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationKind {
    Highlight,
    Spotlight,
    Tooltip,
    Callout,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum HighlightLook {
    Clean,
    Box,
    Circle,
    Underline,
    Marker,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationSpec {
    pub kind: AnnotationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look: Option<HighlightLook>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f64>,
    #[serde(rename = "match")]
    pub match_text: String,
    pub text: String,
    pub color: String,
    pub padding: f64,
    pub lead_ms: f64,
    pub hold_ms: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum KeystrokePosition {
    Bottom,
    Top,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "shape", rename_all = "lowercase")]
pub enum CursorLook {
    Arrow { fill: String, edge: String },
    Mac,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct RippleLook {
    pub color: String,
    pub size: f64,
    pub width: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct KeystrokeLook {
    pub position: KeystrokePosition,
    pub typed: bool,
    pub chords: bool,
    pub keycap: String,
    pub ink: String,
    pub size: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlurArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub strength: f64,
    pub softness: f64,
    pub roundness: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_ms: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct PictureLook {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brightness: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contrast: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saturation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warmth: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vignette: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grain: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CameraTuning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s_max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_min_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordingProps {
    pub src: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_src: Option<String>,
    pub log: ActionLog,
    pub name: String,
    pub speed: f64,
    pub zoom: bool,
    pub tuning: CameraTuning,
    pub motion_blur: bool,
    pub style: CameraStyle,
    pub chrome: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_style: Option<WindowStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_theme: Option<FrameTheme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_bar: Option<bool>,
    pub backdrop: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop_chosen: Option<bool>,
    pub cursor: bool,
    pub cursor_scale: f64,
    pub cursor_look: CursorLook,
    pub ripple: Option<RippleLook>,
    pub extra_beats: Vec<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keystrokes: Option<KeystrokeLook>,
    pub annotations: Vec<AnnotationSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blur_areas: Option<Vec<BlurArea>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look: Option<PictureLook>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen: Option<bool>,
}
