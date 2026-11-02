use opsis_macros::choice;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::actions::{Action, ActionLog};
use crate::values::StyleEntry;

#[choice(export)]
pub enum CameraStyle {
    Classic,
    Proof,
    Ledger,
    Stage,
}

#[choice(export)]
pub enum AddressMode {
    #[label("This address")]
    Fixed,
    #[label("This host with the page's path")]
    Host,
}

#[choice(export)]
pub enum WindowStyle {
    #[label("macOS")]
    Macos,
    #[label("Windows")]
    Windows,
    #[label("Linux")]
    Linux,
}

#[choice(export)]
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

#[choice(export)]
pub enum HighlightLook {
    #[label("Clean")]
    Clean,
    #[label("Hand-drawn box")]
    Box,
    #[label("Circle")]
    Circle,
    #[label("Underline")]
    Underline,
    #[label("Marker")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_look: Option<LabelLook>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_look: Option<AnnotationLook>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationLook {
    pub fade_s: f64,
    pub radius: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub softness: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker_opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draw_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strokes: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CursorFeel {
    pub grow_with_zoom: f64,
    pub outline_width: f64,
    pub shadow: f64,
    pub press_squash: f64,
    pub hide_while_typing: bool,
}

#[choice(export)]
pub enum KeyNames {
    #[default]
    #[label("Symbols (⌘⇧K)")]
    Symbols,
    #[label("Words (Cmd + Shift + K)")]
    Words,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LabelLook {
    pub font_weight: f64,
    pub box_padding: f64,
    pub radius: f64,
    pub pointer_size: f64,
    pub shadow: bool,
    pub arrow_size: f64,
    pub line_width: f64,
    pub tilt: f64,
}

#[choice(export)]
pub enum KeystrokePosition {
    #[label("Bottom")]
    Bottom,
    #[label("Top")]
    Top,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "shape", rename_all = "lowercase")]
pub enum CursorLook {
    Arrow { fill: String, edge: String },
    Mac,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RippleLook {
    pub color: String,
    pub size: f64,
    pub width: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KeystrokeLook {
    pub position: KeystrokePosition,
    pub typed: bool,
    pub chords: bool,
    pub keycap: String,
    pub ink: String,
    pub size: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chord_hold_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typed_hold_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_names: Option<KeyNames>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caret: Option<bool>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct FootageChain {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grade: Vec<StyleEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redact: Vec<StyleEntry>,
}

impl FootageChain {
    pub const fn is_empty(&self) -> bool {
        self.grade.is_empty() && self.redact.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CameraTuning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s_max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_min_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_max_scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_px: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_share: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub off_centre: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join_within_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stay_zoomed_within_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_after_typing_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rest_at_end_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longest_pan_px: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub easing: Option<CameraEasing>,
}

#[choice(export)]
pub enum CameraEasing {
    #[default]
    #[label("Smooth")]
    Smooth,
    #[label("Gentle")]
    Gentle,
    #[label("Snappy")]
    Snappy,
    #[label("Even")]
    Linear,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordingProps {
    pub src: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_mode: Option<AddressMode>,
    pub backdrop: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop_chosen: Option<bool>,
    pub cursor: bool,
    pub cursor_scale: f64,
    pub cursor_look: CursorLook,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_feel: Option<CursorFeel>,
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<RippleLook>)]
    pub ripple: Option<RippleLook>,
    pub extra_beats: Vec<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keystrokes: Option<KeystrokeLook>,
    pub annotations: Vec<AnnotationSpec>,
    #[serde(default, skip_serializing_if = "FootageChain::is_empty")]
    pub footage: FootageChain,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footage_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_fit: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_ground: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rim_glow: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lift_colours: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutter_angle: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bleed_blur: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bleed_dim: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_font: Option<String>,
}
