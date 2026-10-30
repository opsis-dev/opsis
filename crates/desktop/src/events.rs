use opsis_types::edits::DocumentState;
use opsis_types::engine::{EngineInstallEvent, EngineStatus};
use opsis_types::events::{LogLine, RunEvent, RunState, RunView};
use opsis_types::live::LiveEvent;
use opsis_types::project::ProjectFilesChange;
use opsis_types::settings::AppSettings;
use opsis_types::shell::{
    ApiServerStatus, CloseRequest, DockPanelName, MenuCommand, PlaybackCall, Playhead, ReplayTab,
    Selection, SettingsPage, ViewPrefChange, WindowState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri_specta::Event;

use crate::compile::EditorCompile;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct DocumentChanged {
    pub document: DocumentState,
    pub edited: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct DiskChangeAsked {
    pub readable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ProjectReloaded;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct OpenRequested {
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ProjectFilesChanged(pub ProjectFilesChange);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct RunEventFired(pub RunEvent);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct EngineInstallProgress(pub EngineInstallEvent);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct EngineChanged(pub EngineStatus);

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
pub struct LiveWindowWatching(pub bool);

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
pub struct SelectionShared {
    pub from: String,
    pub selection: Selection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct PlayheadShared {
    pub from: String,
    pub playhead: Playhead,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct PlaybackCalled(pub PlaybackCall);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ReplayRequested {
    pub node_id: Option<String>,
    pub tab: ReplayTab,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct RunViewChanged(pub RunView);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct RunLogged(pub LogLine);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct RunStateReset(pub RunState);
