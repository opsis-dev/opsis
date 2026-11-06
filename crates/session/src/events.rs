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

use crate::Session;
use crate::compile::EditorCompile;
use crate::views::ViewEventFired;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DocumentChanged {
    pub document: DocumentState,
    pub edited: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiskChangeAsked {
    pub readable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProjectReloaded;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OpenRequested {
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProjectFilesChanged(pub ProjectFilesChange);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RunEventFired(pub RunEvent);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EngineInstallProgress(pub EngineInstallEvent);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EngineChanged(pub EngineStatus);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SettingsChanged(pub AppSettings);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ViewPrefChanged(pub ViewPrefChange);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ApiStatusChanged(pub ApiServerStatus);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MenuCommandFired(pub MenuCommand);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PanelWindowsChanged(pub Vec<DockPanelName>);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LiveWindowWatching(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ConfirmClose(pub CloseRequest);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FlushSave;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WindowStateChanged(pub WindowState);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SettingsPageShown(pub SettingsPage);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LiveNoted {
    pub capture_id: String,
    pub node_id: String,
    pub t: f64,
    pub event: LiveEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Compiled(#[specta(type = EditorCompile)] pub Value);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SelectionShared {
    pub from: String,
    pub selection: Selection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PlayheadShared {
    pub from: String,
    pub playhead: Playhead,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PlaybackCalled(pub PlaybackCall);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReplayRequested {
    pub node_id: Option<String>,
    pub tab: ReplayTab,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RunViewChanged(pub RunView);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RunLogged(pub LogLine);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RunStateReset(pub RunState);

pub const MAIN_WINDOW: &str = "main";
pub const LIVE_WINDOW: &str = "live";

macro_rules! session_events {
    ($($name:ident),* $(,)?) => {
        #[derive(Debug, Clone)]
        pub enum SessionEvent {
            $($name($name),)*
        }

        $(impl From<$name> for SessionEvent {
            fn from(event: $name) -> Self {
                Self::$name(event)
            }
        })*
    };
}

session_events!(
    DocumentChanged,
    DiskChangeAsked,
    ProjectReloaded,
    OpenRequested,
    ProjectFilesChanged,
    RunEventFired,
    EngineInstallProgress,
    EngineChanged,
    SettingsChanged,
    ViewPrefChanged,
    ApiStatusChanged,
    MenuCommandFired,
    PanelWindowsChanged,
    ConfirmClose,
    FlushSave,
    WindowStateChanged,
    SettingsPageShown,
    LiveNoted,
    LiveWindowWatching,
    Compiled,
    SelectionShared,
    PlayheadShared,
    PlaybackCalled,
    ReplayRequested,
    RunViewChanged,
    RunLogged,
    RunStateReset,
    ViewEventFired,
);

#[derive(Debug, Clone)]
pub struct Addressed {
    pub to: Option<String>,
    pub event: SessionEvent,
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("nothing is listening for the session's events")]
pub struct Unheard;

pub trait Emit {
    fn emit(&self, app: &Session) -> Result<(), Unheard>;
    fn emit_to(&self, app: &Session, window: &str) -> Result<(), Unheard>;
}

impl<T: Clone + Into<SessionEvent>> Emit for T {
    fn emit(&self, app: &Session) -> Result<(), Unheard> {
        send(app, None, self.clone().into())
    }

    fn emit_to(&self, app: &Session, window: &str) -> Result<(), Unheard> {
        send(app, Some(window.to_owned()), self.clone().into())
    }
}

fn send(app: &Session, to: Option<String>, event: SessionEvent) -> Result<(), Unheard> {
    if app.send(Addressed { to, event }) {
        Ok(())
    } else {
        Err(Unheard)
    }
}
