use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

use crate::engine::EngineStatus;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ViewPrefChange {
    pub dir: String,
    pub key: String,
    #[specta(type = specta_typescript::Unknown)]
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub maximized: bool,
    pub fullscreen: bool,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SettingsPage {
    General,
    Projects,
    Engine,
    Recording,
    Fonts,
    Export,
    Api,
    Licences,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DockPanelName {
    Timeline,
    Mixer,
    Run,
}

impl DockPanelName {
    pub const ALL: [Self; 3] = [Self::Timeline, Self::Mixer, Self::Run];

    pub fn id(self) -> &'static str {
        match self {
            Self::Timeline => "timeline",
            Self::Mixer => "mixer",
            Self::Run => "run",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Timeline => "Timeline",
            Self::Mixer => "Mixer",
            Self::Run => "Run",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub enum MenuCommand {
    #[serde(rename = "project:new")]
    ProjectNew,
    #[serde(rename = "project:open")]
    ProjectOpen,
    #[serde(rename = "project:save")]
    ProjectSave,
    #[serde(rename = "edit:undo")]
    EditUndo,
    #[serde(rename = "edit:redo")]
    EditRedo,
    #[serde(rename = "edit:group")]
    EditGroup,
    #[serde(rename = "edit:select-all")]
    EditSelectAll,
    #[serde(rename = "edit:layout")]
    EditLayout,
    #[serde(rename = "edit:add-node")]
    EditAddNode,
    #[serde(rename = "run:start")]
    RunStart,
    #[serde(rename = "run:cancel")]
    RunCancel,
    #[serde(rename = "view:settings")]
    ViewSettings,
    #[serde(rename = "view:projects")]
    ViewProjects,
    #[serde(rename = "view:toggle-left")]
    ViewToggleLeft,
    #[serde(rename = "view:toggle-bottom")]
    ViewToggleBottom,
    #[serde(rename = "view:toggle-right")]
    ViewToggleRight,
    #[serde(rename = "view:toggle-minimap")]
    ViewToggleMinimap,
    #[serde(rename = "view:focus-timeline")]
    ViewFocusTimeline,
    #[serde(rename = "view:focus-mixer")]
    ViewFocusMixer,
    #[serde(rename = "view:focus-run")]
    ViewFocusRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WindowRole {
    Cut,
    Copy,
    Paste,
    SelectAll,
    Reload,
    ToggleDevTools,
    ResetZoom,
    ZoomIn,
    ZoomOut,
    #[serde(rename = "togglefullscreen")]
    ToggleFullscreen,
    Minimize,
    ToggleMaximize,
    Close,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MenuEntry {
    Command {
        label: String,
        command: MenuCommand,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        accelerator: Option<String>,
    },
    Role {
        label: String,
        role: WindowRole,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        accelerator: Option<String>,
    },
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct MenuSection {
    pub label: String,
    pub items: Vec<MenuEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CloseRequest {
    pub ask: bool,
    pub busy: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CloseDecision {
    pub close: bool,
    pub dont_ask: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ApiServerStatus {
    pub enabled: bool,
    pub listening: bool,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppVersions {
    pub app: String,
    pub tauri: String,
    pub webview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnvStatus {
    pub platform: String,
    pub arch: String,
    pub is_packaged: bool,
    pub user_data: String,
    pub versions: AppVersions,
    pub engine: EngineStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub selected_ids: Vec<String>,
    pub active_ids: Vec<String>,
    pub pinned_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Playhead {
    pub frame: f64,
    pub playing: bool,
    pub rate: f64,
    pub fps: u32,
    pub duration_in_frames: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_point: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_point: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "call", rename_all = "camelCase")]
pub enum PlaybackCall {
    Seek { frame: f64 },
    Play,
    Pause,
    Rate { rate: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ReplayTab {
    Replay,
    Trace,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SharedState {
    pub selection: Selection,
    pub playhead: Option<Playhead>,
}
