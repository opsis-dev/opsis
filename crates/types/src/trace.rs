use serde::{Deserialize, Serialize};
use specta::Type;

use crate::actions::{Bounds, Point, Viewport};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraceAction {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub group: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub start: f64,
    pub end: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub logs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point: Option<Point>,
    #[serde(default, rename = "box", skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Bounds>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    Log,
    Info,
    Warning,
    Error,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraceConsole {
    pub level: ConsoleLevel,
    pub text: String,
    pub time: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraceError {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    pub time: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraceRequest {
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub mime_type: String,
    pub resource_type: String,
    pub size: Option<f64>,
    pub start: f64,
    pub duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraceFrame {
    pub src: String,
    pub time: f64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Trace")]
pub struct TraceHeader {
    pub id: String,
    pub node_id: String,
    pub label: String,
    pub ok: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraceModel {
    #[serde(flatten)]
    pub header: TraceHeader,
    pub start: f64,
    pub end: f64,
    pub viewport: Option<Viewport>,
    pub actions: Vec<TraceAction>,
    pub frames: Vec<TraceFrame>,
    pub console: Vec<TraceConsole>,
    pub errors: Vec<TraceError>,
    pub network: Vec<TraceRequest>,
}
