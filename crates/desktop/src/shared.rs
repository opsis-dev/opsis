use std::sync::Mutex;

use opsis_types::shell::{MenuCommand, PlaybackCall, Playhead, ReplayTab, Selection, SharedState};
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_specta::Event;

use crate::error::CommandResult;
use crate::events::{
    MenuCommandFired, PlaybackCalled, PlayheadShared, ReplayRequested, SelectionShared,
};
use crate::windows::MAIN_WINDOW;

#[derive(Default)]
pub struct Shared(Mutex<SharedState>);

impl Shared {
    fn state(&self) -> std::sync::MutexGuard<'_, SharedState> {
        self.0.lock().expect("shared state")
    }
}

pub fn forget_project(app: &AppHandle) {
    app.state::<Shared>().state().selection = Selection::default();
    if let Err(error) = (SelectionShared {
        from: String::new(),
        selection: Selection::default(),
    })
    .emit(app)
    {
        log::warn!("clearing the selection: {error}");
    }
}

#[tauri::command]
#[specta::specta]
pub fn shared_state(shared: State<'_, Shared>) -> SharedState {
    shared.state().clone()
}

#[tauri::command]
#[specta::specta]
pub fn share_selection(
    app: AppHandle,
    window: WebviewWindow,
    shared: State<'_, Shared>,
    selection: Selection,
) -> CommandResult<()> {
    shared.state().selection = selection.clone();
    Ok(SelectionShared {
        from: window.label().to_owned(),
        selection,
    }
    .emit(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn share_playhead(
    app: AppHandle,
    window: WebviewWindow,
    shared: State<'_, Shared>,
    playhead: Playhead,
) -> CommandResult<()> {
    shared.state().playhead = Some(playhead.clone());
    Ok(PlayheadShared {
        from: window.label().to_owned(),
        playhead,
    }
    .emit(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn control_playback(app: AppHandle, call: PlaybackCall) -> CommandResult<()> {
    Ok(PlaybackCalled(call).emit_to(&app, MAIN_WINDOW)?)
}

fn raise_editor(app: &AppHandle) {
    if let Some(main) = app.get_webview_window(MAIN_WINDOW)
        && let Err(error) = main.set_focus()
    {
        log::warn!("raising the editor: {error}");
    }
}

#[tauri::command]
#[specta::specta]
pub fn show_replay(app: AppHandle, node_id: Option<String>, tab: ReplayTab) -> CommandResult<()> {
    raise_editor(&app);
    Ok(ReplayRequested { node_id, tab }.emit_to(&app, MAIN_WINDOW)?)
}

#[tauri::command]
#[specta::specta]
pub fn send_menu_to_editor(app: AppHandle, command: MenuCommand) -> CommandResult<()> {
    raise_editor(&app);
    Ok(MenuCommandFired(command).emit_to(&app, MAIN_WINDOW)?)
}
