use opsis_session::Session;
use opsis_session::events::{Addressed, SessionEvent};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tauri::AppHandle;
use tauri_specta::Event;
use tokio::sync::broadcast::error::RecvError;

use crate::events;

fn emit<E>(app: &AppHandle, to: Option<&str>, payload: &impl Serialize)
where
    E: Event + Serialize + DeserializeOwned + Clone,
{
    let event: E = match serde_json::to_value(payload).and_then(serde_json::from_value) {
        Ok(event) => event,
        Err(error) => {
            log::warn!("reading a session event for the windows: {error}");
            return;
        }
    };
    let sent = match to {
        Some(window) => event.emit_to(app, window),
        None => event.emit(app),
    };
    if let Err(error) = sent {
        log::warn!("sending an event to the windows: {error}");
    }
}

macro_rules! forward_events {
    ($app:expr, $to:expr, $event:expr, $($name:ident),* $(,)?) => {
        match $event {
            $(SessionEvent::$name(payload) => emit::<events::$name>($app, $to, &payload),)*
        }
    };
}

fn forward(app: &AppHandle, addressed: Addressed) {
    let to = addressed.to.as_deref();
    forward_events!(
        app,
        to,
        addressed.event,
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
}

pub fn start(app: &AppHandle, session: &Session) {
    let mut events = session.subscribe();
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(addressed) => forward(&app, addressed),
                Err(RecvError::Lagged(missed)) => {
                    log::warn!("{missed} session events were dropped before reaching the windows");
                }
                Err(RecvError::Closed) => break,
            }
        }
    });
}
