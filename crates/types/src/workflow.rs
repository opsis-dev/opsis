use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

use crate::actions::Viewport;
use crate::fonts::ProjectFonts;
use crate::mix::Mix;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum PortType {
    BrowserSession,
    Recording,
    RecordingStyle,
    Grade,
    Clip,
    Audio,
    Captions,
    Image,
    Media,
    FileInfo,
    Folder,
}

impl PortType {
    pub const ALL: [PortType; 11] = [
        Self::BrowserSession,
        Self::Recording,
        Self::RecordingStyle,
        Self::Grade,
        Self::Clip,
        Self::Audio,
        Self::Captions,
        Self::Image,
        Self::Media,
        Self::FileInfo,
        Self::Folder,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::BrowserSession => "browser-session",
            Self::Recording => "recording",
            Self::RecordingStyle => "recording-style",
            Self::Grade => "grade",
            Self::Clip => "clip",
            Self::Audio => "audio",
            Self::Captions => "captions",
            Self::Image => "image",
            Self::Media => "media",
            Self::FileInfo => "file-info",
            Self::Folder => "folder",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::BrowserSession => "browser session",
            Self::Recording => "recording",
            Self::RecordingStyle => "style",
            Self::Grade => "grade",
            Self::Clip => "clip",
            Self::Audio => "sound",
            Self::Captions => "captions",
            Self::Image => "image",
            Self::Media => "file",
            Self::FileInfo => "file info",
            Self::Folder => "folder",
        }
    }
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
pub enum Theme {
    Light,
    #[default]
    Dark,
}

impl Theme {
    pub const ALL: [Self; 2] = [Self::Light, Self::Dark];

    pub fn background(self) -> &'static str {
        match self {
            Self::Light => "#fafafa",
            Self::Dark => "#09090b",
        }
    }
}

fn default_width() -> u32 {
    1920
}

fn default_height() -> u32 {
    1080
}

fn default_fps() -> u32 {
    30
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct WorkflowSettings {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_fps")]
    pub fps: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub fonts: ProjectFonts,
}

impl Default for WorkflowSettings {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            fps: default_fps(),
            background: None,
            theme: Theme::Dark,
            fonts: ProjectFonts::default(),
        }
    }
}

pub const DEFAULT_VIEWPORT: Viewport = Viewport {
    width: 1920,
    height: 1080,
};
pub const DEFAULT_TAKE_MS: f64 = 6000.0;
pub const DEFAULT_STILL_MS: f64 = 3000.0;
pub const WORKFLOW_FILE: &str = "project.opsis.json";

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub position: Position,
    #[serde(default)]
    #[specta(type = crate::JsonObject)]
    pub params: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Viewport>,
}

impl WorkflowNode {
    pub fn is_disabled(&self) -> bool {
        self.disabled == Some(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEdge {
    pub id: String,
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Marker {
    pub id: String,
    pub at_ms: f64,
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Workflow {
    pub version: u8,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub settings: WorkflowSettings,
    #[serde(default)]
    pub variables: indexmap::IndexMap<String, String>,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mix: Option<Mix>,
    #[serde(default)]
    pub markers: Vec<Marker>,
}

impl Workflow {
    pub fn node(&self, id: &str) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn normalized(mut self) -> Self {
        if let Some(mix) = self.mix.as_mut() {
            let ids: std::collections::HashSet<&str> =
                self.nodes.iter().map(|n| n.id.as_str()).collect();
            mix.channels.retain(|id, _| ids.contains(id.as_str()));
        }
        self
    }
}
