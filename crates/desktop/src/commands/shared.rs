use opsis_session::error::CommandResult;
use opsis_types::shell::{MenuCommand, PlaybackCall, Playhead, ReplayTab, Selection, SharedState};
use tauri::{AppHandle, WebviewWindow};

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub fn shared_state(app: AppHandle) -> SharedState {
    let session = session_of(&app);
    opsis_session::shared::shared_state(&session)
}

#[tauri::command]
#[specta::specta]
pub fn share_selection(
    app: AppHandle,
    window: WebviewWindow,
    selection: Selection,
) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::shared::share_selection(&session, window.label(), selection)
}

#[tauri::command]
#[specta::specta]
pub fn share_playhead(
    app: AppHandle,
    window: WebviewWindow,
    playhead: Playhead,
) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::shared::share_playhead(&session, window.label(), playhead)
}

#[tauri::command]
#[specta::specta]
pub fn control_playback(app: AppHandle, call: PlaybackCall) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::shared::control_playback(&session, call)
}

#[tauri::command]
#[specta::specta]
pub fn show_replay(app: AppHandle, node_id: Option<String>, tab: ReplayTab) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::shared::show_replay(&session, node_id, tab)
}

#[tauri::command]
#[specta::specta]
pub fn send_menu_to_editor(app: AppHandle, command: MenuCommand) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::shared::send_menu_to_editor(&session, command)
}
