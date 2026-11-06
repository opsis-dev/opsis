use opsis_session::error::CommandResult;
use opsis_types::edits::{
    DocumentState, Edited, Removal, TimelineEdit, TimelineReply, WorkflowEdit,
};
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub fn document_state(app: AppHandle) -> Option<DocumentState> {
    let session = session_of(&app);
    opsis_session::document::document_state(&session)
}

#[tauri::command]
#[specta::specta]
pub fn edit_workflow(app: AppHandle, edit: WorkflowEdit) -> CommandResult<Edited> {
    let session = session_of(&app);
    opsis_session::document::edit_workflow(&session, edit)
}

#[tauri::command]
#[specta::specta]
pub fn undo(app: AppHandle) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::document::undo(&session)
}

#[tauri::command]
#[specta::specta]
pub fn redo(app: AppHandle) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::document::redo(&session)
}

#[tauri::command]
#[specta::specta]
pub fn begin_gesture(app: AppHandle) {
    let session = session_of(&app);
    opsis_session::document::begin_gesture(&session);
}

#[tauri::command]
#[specta::specta]
pub fn end_gesture(app: AppHandle) {
    let session = session_of(&app);
    opsis_session::document::end_gesture(&session);
}

#[tauri::command]
#[specta::specta]
pub fn edit_timeline(app: AppHandle, edit: TimelineEdit) -> CommandResult<TimelineReply> {
    let session = session_of(&app);
    opsis_session::document::edit_timeline(&session, edit)
}

#[tauri::command]
#[specta::specta]
pub fn timeline_removal(app: AppHandle, selected: Vec<String>) -> Option<Removal> {
    let session = session_of(&app);
    opsis_session::document::timeline_removal(&session, &selected)
}

#[tauri::command]
#[specta::specta]
pub async fn save_project(app: AppHandle) -> CommandResult<bool> {
    let session = session_of(&app);
    opsis_session::document::save_project(session.clone()).await
}

#[tauri::command]
#[specta::specta]
pub async fn resolve_disk_change(
    app: AppHandle,
    choice: opsis_session::document::DiskChoice,
) -> CommandResult<bool> {
    let session = session_of(&app);
    opsis_session::document::resolve_disk_change(session.clone(), choice).await
}
