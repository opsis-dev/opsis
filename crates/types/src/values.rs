use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

use crate::actions::{ActionLog, Bounds, Viewport};
use crate::terminal::{TerminalSessionValue, TerminalTheme};
use crate::timeline::{AudioItem, TimelineTrack};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StyleEntry {
    pub node_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[specta(type = crate::JsonObject)]
    pub params: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordingValue {
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<ActionLog>)]
    pub log: Option<ActionLog>,
    pub duration_ms: f64,
    pub node_id: String,
    pub edits: Vec<StyleEntry>,
    pub source: TakeSource,
}

impl RecordingValue {
    pub const fn kind(&self) -> TakeKind {
        self.source.kind()
    }

    pub fn file(&self) -> Option<&str> {
        match &self.source {
            TakeSource::Browser(b) => b.video.as_deref(),
            TakeSource::Terminal(t) => t.cast.as_deref(),
        }
    }

    pub fn video(&self) -> Option<&str> {
        match &self.source {
            TakeSource::Browser(b) => b.video.as_deref(),
            TakeSource::Terminal(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum TakeSource {
    Browser(BrowserSource),
    Terminal(TerminalSource),
}

impl TakeSource {
    pub const fn kind(&self) -> TakeKind {
        match self {
            Self::Browser(_) => TakeKind::Browser,
            Self::Terminal(_) => TakeKind::Terminal,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum TakeKind {
    Browser,
    Terminal,
}

impl TakeKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Terminal => "terminal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSource {
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<String>)]
    pub video: Option<String>,
    pub viewport: Viewport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSource {
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<String>)]
    pub cast: Option<String>,
    pub cols: u16,
    pub rows: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Box<TerminalTheme>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TaggedRecording {
    Recording(RecordingValue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct ImageValue {
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<String>)]
    pub src: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Bounds>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Video,
    Audio,
    Image,
    #[serde(rename = "terminal-take")]
    TerminalTake,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MediaValue {
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<String>)]
    pub src: Option<String>,
    pub kind: MediaKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_audio: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClipValue {
    pub duration_in_frames: u32,
    pub tracks: Vec<TimelineTrack>,
    pub audio: Vec<AudioItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording: Option<TaggedRecording>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioValue {
    pub items: Vec<AudioItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_in_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hits: Option<Hits>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Hits {
    pub clicks: u32,
    pub keys: u32,
    #[serde(default)]
    pub transitions: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Caption {
    pub text: String,
    pub start_ms: f64,
    pub end_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct CaptionsValue {
    pub captions: Vec<Caption>,
}

macro_rules! browser_actions {
    ($($action:ident = $kind:literal),+ $(,)?) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
        )]
        pub enum BrowserAction {
            $(#[serde(rename = $kind)] $action),+
        }

        impl BrowserAction {
            pub const ALL: &'static [Self] = &[$(Self::$action),+];

            pub const fn kind(self) -> &'static str {
                match self {
                    $(Self::$action => $kind),+
                }
            }
        }
    };
}

browser_actions! {
    StartRecording = "browser.startRecording",
    Navigate = "browser.navigate",
    Click = "browser.click",
    Type = "browser.type",
    Hover = "browser.hover",
    Drag = "browser.drag",
    Scroll = "browser.scroll",
    Select = "browser.select",
    Upload = "browser.upload",
    Press = "browser.press",
    Wait = "browser.wait",
    Assert = "browser.assert",
    Hoist = "browser.hoist",
    SetState = "browser.setState",
    MockRoute = "browser.mockRoute",
    FreezeClock = "browser.freezeClock",
    HideElements = "browser.hideElements",
    Chapter = "browser.chapter",
    OverlayHtml = "browser.overlayHtml",
    SignIn = "browser.signIn",
}

impl BrowserAction {
    pub fn of_kind(kind: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.kind() == kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct SessionStep {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: BrowserAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[specta(type = crate::JsonObject)]
    pub params: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionBrowser {
    pub node_id: String,
    #[specta(type = crate::JsonObject)]
    pub params: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct BrowserSessionValue {
    pub browser: SessionBrowser,
    pub steps: Vec<SessionStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct EntriesValue {
    pub entries: Vec<StyleEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CompileValue {
    BrowserSession(BrowserSessionValue),
    TerminalSession(TerminalSessionValue),
    Recording(RecordingValue),
    RecordingStyle(EntriesValue),
    Grade(EntriesValue),
    Image(ImageValue),
    Media(MediaValue),
    Clip(ClipValue),
    Audio(AudioValue),
    Captions(CaptionsValue),
}

impl CompileValue {
    pub const fn port_type(&self) -> crate::workflow::PortType {
        use crate::workflow::PortType;
        match self {
            Self::BrowserSession(_) => PortType::BrowserSession,
            Self::TerminalSession(_) => PortType::TerminalSession,
            Self::Recording(_) => PortType::Recording,
            Self::RecordingStyle(_) => PortType::RecordingStyle,
            Self::Grade(_) => PortType::Grade,
            Self::Image(_) => PortType::Image,
            Self::Media(_) => PortType::Media,
            Self::Clip(_) => PortType::Clip,
            Self::Audio(_) => PortType::Audio,
            Self::Captions(_) => PortType::Captions,
        }
    }

    pub const fn as_clip(&self) -> Option<&ClipValue> {
        match self {
            Self::Clip(clip) => Some(clip),
            _ => None,
        }
    }

    pub const fn as_recording(&self) -> Option<&RecordingValue> {
        match self {
            Self::Recording(recording) => Some(recording),
            _ => None,
        }
    }

    pub const fn as_session(&self) -> Option<&BrowserSessionValue> {
        match self {
            Self::BrowserSession(session) => Some(session),
            _ => None,
        }
    }

    pub const fn as_style(&self) -> Option<&EntriesValue> {
        match self {
            Self::RecordingStyle(style) => Some(style),
            _ => None,
        }
    }

    pub const fn as_grade(&self) -> Option<&EntriesValue> {
        match self {
            Self::Grade(grade) => Some(grade),
            _ => None,
        }
    }

    pub const fn as_image(&self) -> Option<&ImageValue> {
        match self {
            Self::Image(image) => Some(image),
            _ => None,
        }
    }

    pub const fn as_media(&self) -> Option<&MediaValue> {
        match self {
            Self::Media(media) => Some(media),
            _ => None,
        }
    }

    pub const fn as_audio(&self) -> Option<&AudioValue> {
        match self {
            Self::Audio(audio) => Some(audio),
            _ => None,
        }
    }

    pub const fn as_captions(&self) -> Option<&CaptionsValue> {
        match self {
            Self::Captions(captions) => Some(captions),
            _ => None,
        }
    }

    pub fn src(&self) -> Option<&str> {
        match self {
            Self::Recording(v) => v.file(),
            Self::Image(v) => v.src.as_deref(),
            Self::Media(v) => v.src.as_deref(),
            _ => None,
        }
    }
}

impl NodeResult {
    pub fn output(&self, port: &str) -> Option<&CompileValue> {
        self.outputs.get(port)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Video,
    Image,
    Audio,
    Json,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stamp: Option<String>,
}

impl Artifact {
    pub fn labelled(kind: ArtifactKind, path: String, label: impl Into<String>) -> Self {
        Self {
            kind,
            path,
            label: Some(label.into()),
            bytes: None,
            stamp: None,
        }
    }
}

impl Artifact {
    pub fn name(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.path)
    }
}

pub type PortValues = IndexMap<String, CompileValue>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Measured {
    NoiseLevel {
        before_db: f64,
        after_db: f64,
    },
    File {
        bytes: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fps: Option<f64>,
    },
    Changes {
        count: u32,
    },
    Fitted {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tempo: Option<f64>,
        loops: u32,
    },
    Tape {
        steps: u32,
        ignored: Vec<String>,
        warnings: Vec<String>,
    },
    Cast {
        events: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeResult {
    pub hash: String,
    pub outputs: PortValues,
    pub artifacts: Vec<Artifact>,
    pub finished_at: i64,
    pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub measured: Vec<Measured>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeFailure {
    pub message: String,
    pub at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LastRun {
    pub updated_at: i64,
    pub nodes: IndexMap<String, NodeResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failures: Option<IndexMap<String, NodeFailure>>,
}

impl LastRun {
    pub fn parse(value: &Value) -> Option<Self> {
        let object = value.as_object()?;
        let updated_at = object.get("updatedAt")?.as_i64()?;
        let entries = |key: &str| -> Option<IndexMap<String, Value>> {
            object
                .get(key)
                .and_then(Value::as_object)
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        };
        let nodes = entries("nodes")?
            .into_iter()
            .filter_map(|(k, v)| serde_json::from_value::<NodeResult>(v).ok().map(|r| (k, r)))
            .collect();
        let failures = match object.get("failures") {
            None => None,
            Some(_) => Some(
                entries("failures")?
                    .into_iter()
                    .filter_map(|(k, v)| {
                        serde_json::from_value::<NodeFailure>(v)
                            .ok()
                            .map(|f| (k, f))
                    })
                    .collect(),
            ),
        };
        Some(Self {
            updated_at,
            nodes,
            failures,
        })
    }
}
