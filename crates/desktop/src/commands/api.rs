use opsis_types::shell::ApiServerStatus;
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub fn api_status(app: AppHandle) -> ApiServerStatus {
    let session = session_of(&app);
    opsis_session::api::api_status(&session)
}
