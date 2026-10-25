use serde::{Deserialize, Serialize};
use specta::Type;

pub const ENGINE_MISSING_MESSAGE: &str =
    "The video engine isn't set up yet. Set it up in Settings → Engine.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub dir: String,
    pub headless_shell: Option<String>,
    pub ffmpeg: Option<String>,
    pub ffprobe: Option<String>,
    pub complete: bool,
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
