use serde::{Deserialize, Serialize};
use specta::Type;

pub const ENGINE_MISSING_MESSAGE: &str =
    "The video engine isn't set up yet. Set it up in Settings → Engine.";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum EngineTool {
    Browser,
    Ffmpeg,
}

impl EngineTool {
    pub const ALL: [Self; 2] = [Self::Browser, Self::Ffmpeg];

    pub fn label(self) -> &'static str {
        match self {
            Self::Browser => "recording browser",
            Self::Ffmpeg => "video tool",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckedTool {
    pub path: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub modified_ms: i64,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    pub checking: bool,
}

impl ToolStatus {
    pub fn ready(&self) -> bool {
        self.path.is_some() && self.problem.is_none() && !self.checking
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub download_dir: String,
    pub browser: ToolStatus,
    pub ffmpeg: ToolStatus,
    pub complete: bool,
}

impl EngineStatus {
    pub fn tool(&self, tool: EngineTool) -> &ToolStatus {
        match tool {
            EngineTool::Browser => &self.browser,
            EngineTool::Ffmpeg => &self.ffmpeg,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum EnginePart {
    Ffmpeg,
    HeadlessShell,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct EngineInstallUpdate {
    pub part: EnginePart,
    pub step: String,
    pub progress: Option<f64>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct EngineInstallEvent {
    #[serde(flatten)]
    pub update: EngineInstallUpdate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LicenceEntry {
    pub id: String,
    pub title: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}
