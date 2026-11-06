use opsis_session::error::CommandResult;
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub async fn analyze(
    app: AppHandle,
    request: opsis_session::analysis::AnalysisRequest,
) -> CommandResult<opsis_session::analysis::AnalysisResult> {
    let session = session_of(&app);
    opsis_session::analysis::analyze(&session, request).await
}
