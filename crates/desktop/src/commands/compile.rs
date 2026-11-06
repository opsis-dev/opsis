use opsis_compile::catalog::EditorCatalog;
use opsis_session::error::CommandResult;
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub fn set_compile_focus(app: AppHandle, selected: Option<String>, inspected: Option<String>) {
    let session = session_of(&app);
    opsis_session::compile::set_compile_focus(&session, selected, inspected);
}

#[tauri::command]
#[specta::specta]
pub async fn latest_compile(
    app: AppHandle,
) -> CommandResult<Option<opsis_session::compile::EditorCompile>> {
    let session = session_of(&app);
    opsis_session::compile::latest_compile(session.clone()).await
}

#[tauri::command]
#[specta::specta]
pub fn editor_catalog() -> EditorCatalog {
    opsis_session::compile::editor_catalog()
}
