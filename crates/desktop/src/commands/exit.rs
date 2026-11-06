use opsis_types::shell::CloseDecision;
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub fn report_drafts(app: AppHandle, held: bool) {
    let session = session_of(&app);
    opsis_session::exit::report_drafts(&session, held);
}

#[tauri::command]
#[specta::specta]
pub fn close_shown(app: AppHandle) {
    let session = session_of(&app);
    opsis_session::exit::close_shown(&session);
}

#[tauri::command]
#[specta::specta]
pub fn decide_close(app: AppHandle, decision: CloseDecision) {
    let session = session_of(&app);
    opsis_session::exit::decide_close(&session, decision);
}

#[tauri::command]
#[specta::specta]
pub fn save_flushed(app: AppHandle) {
    let session = session_of(&app);
    opsis_session::exit::save_flushed(&session);
}
