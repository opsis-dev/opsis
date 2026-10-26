use opsis_types::engine::EngineInstallEvent;
use opsis_types::events::RunEvent;
use opsis_types::live::LiveEvent;
use opsis_types::project::{ProjectFileChange, ProjectFilesChange};
use opsis_types::settings::AppSettings;
use opsis_types::shell::{
    ApiServerStatus, CloseRequest, DockPanelName, MenuCommand, SettingsPage, ViewPrefChange,
    WindowState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri_specta::Event;

use crate::compile::EditorCompile;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ProjectFileChanged(pub ProjectFileChange);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ProjectFilesChanged(pub ProjectFilesChange);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct RunEventFired(pub RunEvent);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct EngineInstallProgress(pub EngineInstallEvent);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct SettingsChanged(pub AppSettings);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ViewPrefChanged(pub ViewPrefChange);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ApiStatusChanged(pub ApiServerStatus);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct MenuCommandFired(pub MenuCommand);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct PanelWindowsChanged(pub Vec<DockPanelName>);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ConfirmClose(pub CloseRequest);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct FlushSave;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct WindowStateChanged(pub WindowState);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct SettingsPageShown(pub SettingsPage);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct LiveNoted {
    pub capture_id: String,
    pub node_id: String,
    pub t: f64,
    pub event: LiveEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct Compiled(#[specta(type = EditorCompile)] pub Value);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct Relayed {
    pub from: String,
    #[specta(type = specta_typescript::Unknown)]
    pub message: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ApiRequestReceived {
    pub id: String,
    #[specta(type = opsis_api::bodies::EditorRequest)]
    pub request: Value,
}
