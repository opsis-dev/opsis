use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

use crate::actions::Viewport;
use crate::fonts::{FontFamilyDef, FontRole, ProjectFonts};
use crate::mix::{Comp, DuckSettings, Eq, Highpass, Limiter, LoudnessTarget};
use crate::timeline::{Ease, EnvelopeKey};
use crate::workflow::{Position, Theme, Workflow, WorkflowEdge, WorkflowNode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EdgeEnds {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

impl EdgeEnds {
    pub fn id(&self) -> String {
        format!(
            "{}.{}->{}.{}",
            self.source, self.source_handle, self.target, self.target_handle
        )
    }

    pub fn edge(self) -> WorkflowEdge {
        WorkflowEdge {
            id: self.id(),
            source: self.source,
            source_handle: self.source_handle,
            target: self.target,
            target_handle: self.target_handle,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NodePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Viewport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[specta(type = Option<crate::JsonObject>)]
    pub params: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fps: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<ProjectFonts>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StripPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highpass: Option<Highpass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq: Option<Eq>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comp: Option<Comp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fader: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solo: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bus: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BusPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highpass: Option<Highpass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq: Option<Eq>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comp: Option<Comp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fader: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solo: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duck: Option<DuckSettings>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MasterPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fader: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq: Option<Eq>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comp: Option<Comp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limiter: Option<Limiter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loudness: Option<LoudnessTarget>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum MixEdit {
    Channels { ids: Vec<String>, patch: StripPatch },
    ResetChannels { ids: Vec<String> },
    Bus { id: String, patch: BusPatch },
    ResetBus { id: String },
    AddBus,
    RemoveBus { id: String },
    Master { patch: MasterPatch },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum MarkerEdit {
    #[serde(rename_all = "camelCase")]
    Add {
        at_ms: f64,
    },
    #[serde(rename_all = "camelCase")]
    Update {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at_ms: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    Remove {
        id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum WorkflowEdit {
    AddNode {
        #[serde(rename = "type")]
        kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        position: Option<Position>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[specta(type = Option<crate::JsonObject>)]
        params: Option<Map<String, Value>>,
    },
    UpdateNode {
        id: String,
        patch: NodePatch,
    },
    MoveNodes {
        positions: IndexMap<String, Position>,
    },
    RemoveNodes {
        ids: Vec<String>,
    },
    AddEdge {
        ends: EdgeEnds,
    },
    RemoveEdges {
        ids: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    RemoveParts {
        node_ids: Vec<String>,
        edge_ids: Vec<String>,
    },
    SetSettings {
        patch: SettingsPatch,
    },
    AddProjectFont {
        font: FontFamilyDef,
    },
    SetFontRole {
        role: FontRole,
        family: String,
    },
    SetVariables {
        variables: IndexMap<String, String>,
    },
    RenameVariable {
        from: String,
        to: String,
    },
    Rename {
        name: String,
    },
    Paste {
        nodes: Vec<WorkflowNode>,
        edges: Vec<WorkflowEdge>,
        offset: Position,
    },
    Group {
        ids: Vec<String>,
        title: String,
        sizes: IndexMap<String, Viewport>,
    },
    Ungroup {
        id: String,
    },
    Mix {
        edit: MixEdit,
    },
    Markers {
        edit: MarkerEdit,
    },
    Keyframes {
        edit: KeyframeEdit,
    },
    Replace {
        workflow: Box<Workflow>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum KeyframeEdit {
    AddKey {
        node_id: String,
        track: EnvelopeKey,
        frame: f64,
    },
    AppendKey {
        node_id: String,
        track: EnvelopeKey,
    },
    SetKey {
        node_id: String,
        track: EnvelopeKey,
        index: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        frame: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease: Option<Ease>,
    },
    RemoveKey {
        node_id: String,
        track: EnvelopeKey,
        index: u32,
    },
    AddCameraKey {
        node_id: String,
        frame: f64,
    },
    SetCameraKey {
        node_id: String,
        index: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        frame: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        yaw: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pitch: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        distance: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease: Option<Ease>,
    },
    RemoveCameraKey {
        node_id: String,
        index: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Edited {
    pub revision: u64,
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentState {
    pub session: String,
    pub dir: String,
    pub workflow: Workflow,
    pub revision: u64,
    pub saved_revision: u64,
    pub saving: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum TimelineEdit {
    Move {
        bar: String,
        delta: f64,
        ripple: bool,
    },
    TrimStart {
        bar: String,
        delta: f64,
    },
    TrimEnd {
        bar: String,
        delta: f64,
        ripple: bool,
    },
    Slip {
        bar: String,
        delta: f64,
    },
    Speed {
        bar: String,
        delta: f64,
    },
    RollStart {
        bar: String,
        delta: f64,
    },
    RollEnd {
        bar: String,
        delta: f64,
    },
    AudioMove {
        bar: String,
        delta: f64,
        ripple: bool,
    },
    AudioTrimStart {
        bar: String,
        delta: f64,
    },
    AudioTrimEnd {
        bar: String,
        delta: f64,
    },
    Volume {
        bar: String,
        volume: f64,
    },
    Split {
        at: f64,
        selected: Vec<String>,
    },
    SplitBar {
        bar: String,
        at: f64,
    },
    CameraKey {
        key: String,
        delta: f64,
    },
    Keyframes {
        edit: KeyframeEdit,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimelineReply {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Removal {
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<String>,
}
