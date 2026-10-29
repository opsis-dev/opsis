use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

use crate::actions::{Rect, Viewport};
use crate::fonts::FilmFonts;
use crate::recording::RecordingProps;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransitionRef {
    pub kind: String,
    pub duration_in_frames: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[specta(type = Option<crate::JsonObject>)]
    pub params: Option<Map<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub easing: Option<TransitionEase>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionEase {
    #[default]
    Linear,
    In,
    Out,
    InOut,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Ease {
    #[default]
    Linear,
    In,
    Out,
    InOut,
    Hold,
}

impl Ease {
    pub const ALL: [Ease; 5] = [Self::Linear, Self::In, Self::Out, Self::InOut, Self::Hold];

    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Steady",
            Self::In => "Speed up",
            Self::Out => "Slow down",
            Self::InOut => "Smooth",
            Self::Hold => "Jump",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum EnvelopeKey {
    Opacity,
    X,
    Y,
    Scale,
    Volume,
    Pan,
}

impl EnvelopeKey {
    pub const ALL: [Self; 6] = [
        Self::Opacity,
        Self::X,
        Self::Y,
        Self::Scale,
        Self::Volume,
        Self::Pan,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Opacity => "opacity",
            Self::X => "x",
            Self::Y => "y",
            Self::Scale => "scale",
            Self::Volume => "volume",
            Self::Pan => "pan",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub struct DialRange {
    pub min: f64,
    pub max: f64,
}

pub const CAMERA_YAW: DialRange = DialRange {
    min: -180.0,
    max: 180.0,
};
pub const CAMERA_PITCH: DialRange = DialRange {
    min: -89.0,
    max: 89.0,
};
pub const CAMERA_DISTANCE: DialRange = DialRange { min: 0.3, max: 4.0 };
pub const CAMERA_HEIGHT: DialRange = DialRange {
    min: -2.0,
    max: 2.0,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Keyframe {
    #[schemars(range(min = 0))]
    pub frame: f64,
    pub value: f64,
    #[serde(default)]
    pub ease: Ease,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct VisualEnvelope {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Vec<Keyframe>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<Vec<Keyframe>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<Vec<Keyframe>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Vec<Keyframe>>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct AudioEnvelope {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<Vec<Keyframe>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan: Option<Vec<Keyframe>>,
}

fn in_out() -> Ease {
    Ease::InOut
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct CameraKey {
    #[schemars(range(min = 0))]
    pub frame: f64,
    #[serde(default)]
    #[schemars(range(min = CAMERA_YAW.min, max = CAMERA_YAW.max))]
    pub yaw: f64,
    #[serde(default)]
    #[schemars(range(min = CAMERA_PITCH.min, max = CAMERA_PITCH.max))]
    pub pitch: f64,
    #[serde(default = "one")]
    #[schemars(range(min = CAMERA_DISTANCE.min, max = CAMERA_DISTANCE.max))]
    pub distance: f64,
    #[serde(default)]
    #[schemars(range(min = CAMERA_HEIGHT.min, max = CAMERA_HEIGHT.max))]
    pub height: f64,
    #[serde(default = "in_out")]
    pub ease: Ease,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct CameraPath {
    #[serde(default)]
    pub keys: Vec<CameraKey>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ParamUnit {
    Ms,
    Frames,
    Rate,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "i8", try_from = "i8")]
pub enum Sign {
    Same,
    Opposite,
}

impl From<Sign> for i8 {
    fn from(sign: Sign) -> Self {
        match sign {
            Sign::Same => 1,
            Sign::Opposite => -1,
        }
    }
}

impl TryFrom<i8> for Sign {
    type Error = String;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Same),
            -1 => Ok(Self::Opposite),
            other => Err(format!("a sign is 1 or -1, not {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkedParam {
    pub param: String,
    #[specta(type = i8)]
    #[schemars(with = "i8")]
    pub sign: Sign,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub if_non_zero: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParamRef {
    pub node_id: String,
    pub param: String,
    pub unit: ParamUnit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked: Option<Vec<LinkedParam>>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EditHandles {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "move")]
    pub move_to: Option<ParamRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim_start: Option<ParamRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim_end: Option<ParamRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<ParamRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<ParamRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub node_id: String,
    pub port: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct ElementRef {
    pub component: String,
    #[specta(type = crate::JsonObject)]
    pub props: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct SequencerProps {
    pub component: String,
    #[specta(type = crate::JsonObject)]
    pub props: Map<String, Value>,
    pub backgrounds: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroupProps {
    pub tracks: Vec<TimelineTrack>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_from: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remap: Option<Vec<(f64, f64)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shift_pitch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_every: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<Rect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop_fill: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrapper: Option<ElementRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design: Option<Viewport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequencer: Option<SequencerProps>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Design {
    pub width: u32,
    pub height: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemBase {
    pub id: String,
    pub from: u32,
    pub duration_in_frames: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enter: Option<TransitionRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit: Option<TransitionRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handles: Option<EditHandles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<VisualEnvelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<CameraPath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design: Option<Design>,
}

impl ItemBase {
    pub fn new(id: impl Into<String>, from: u32, duration_in_frames: u32) -> Self {
        Self {
            id: id.into(),
            from,
            duration_in_frames,
            enter: None,
            exit: None,
            node_id: None,
            handles: None,
            slot: None,
            envelope: None,
            camera: None,
            design: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TimelineItem {
    Group {
        #[serde(flatten)]
        base: ItemBase,
        props: Box<GroupProps>,
    },
    Recording {
        #[serde(flatten)]
        base: ItemBase,
        props: Box<RecordingProps>,
    },
    Element {
        #[serde(flatten)]
        base: ItemBase,
        component: String,
        #[specta(type = crate::JsonObject)]
        props: Map<String, Value>,
    },
}

impl TimelineItem {
    pub fn base(&self) -> &ItemBase {
        match self {
            Self::Group { base, .. }
            | Self::Recording { base, .. }
            | Self::Element { base, .. } => base,
        }
    }

    pub fn base_mut(&mut self) -> &mut ItemBase {
        match self {
            Self::Group { base, .. }
            | Self::Recording { base, .. }
            | Self::Element { base, .. } => base,
        }
    }

    pub fn group(&self) -> Option<&GroupProps> {
        match self {
            Self::Group { props, .. } => Some(props),
            _ => None,
        }
    }

    pub fn recording(&self) -> Option<&RecordingProps> {
        match self {
            Self::Recording { props, .. } => Some(props),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct TimelineTrack {
    pub id: String,
    pub items: Vec<TimelineItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Duck {
    pub level: f64,
    pub fade_frames: u32,
    pub keys: Vec<String>,
    pub spans: Vec<(f64, f64)>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum AudioRole {
    Music,
    Effect,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum FadeCurve {
    #[default]
    Tri,
    Qsin,
    Exp,
    Log,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub node_id: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub clip_end: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub keep_end: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioItem {
    pub id: String,
    pub src: String,
    pub from: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_in_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim_before: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "loop")]
    pub looped: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_in_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_out_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_curve: Option<FadeCurve>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_track: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<AudioRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_music: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gain: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duck: Option<Duck>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<AudioEnvelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handles: Option<EditHandles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
}

impl AudioItem {
    pub fn new(id: impl Into<String>, src: impl Into<String>, from: i32) -> Self {
        Self {
            id: id.into(),
            src: src.into(),
            from,
            duration_in_frames: None,
            volume: None,
            trim_before: None,
            looped: None,
            rate: None,
            pitch: None,
            hit: None,
            fade_in_frames: None,
            fade_out_frames: None,
            fade_curve: None,
            node_id: None,
            channel: None,
            layer_track: None,
            role: None,
            length_frames: None,
            keep_music: None,
            gain: None,
            duck: None,
            envelope: None,
            handles: None,
            anchor: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct MasterStem {
    pub src: String,
    pub hash: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct SpecAssets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_base: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versions: Option<IndexMap<String, String>>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ReframeMode {
    #[default]
    Follow,
    Center,
    Fit,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum PreviewQuality {
    Fast,
    Exact,
}

impl PreviewQuality {
    pub const ALL: [Self; 2] = [Self::Fast, Self::Exact];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimelineSpec {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_in_frames: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    pub tracks: Vec<TimelineTrack>,
    #[serde(default)]
    pub audio: Vec<AudioItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<SpecAssets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reframe: Option<ReframeMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<PreviewQuality>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<FilmFonts>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mix: Option<MasterStem>,
}
