use opsis_session::error::CommandResult;
use opsis_types::engine::EngineStatus;
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub async fn install_engine(app: AppHandle) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::engine::install_engine(session.clone()).await
}

#[tauri::command]
#[specta::specta]
pub async fn choose_browser(app: AppHandle) -> CommandResult<bool> {
    let session = session_of(&app);
    opsis_session::engine::choose_browser(session.clone()).await
}

#[tauri::command]
#[specta::specta]
pub fn get_engine_status(app: AppHandle) -> EngineStatus {
    let session = session_of(&app);
    opsis_session::engine::get_engine_status(&session)
}
