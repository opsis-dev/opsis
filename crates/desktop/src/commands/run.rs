use opsis_session::error::CommandResult;
use opsis_types::events::{RunState, RunTargets};
use tauri::AppHandle;

use crate::events::LiveNoted;
use crate::frames::FrameChannel;
use crate::session_of;

#[tauri::command]
#[specta::specta]
pub async fn run_workflow(
    app: AppHandle,
    targets: RunTargets,
) -> CommandResult<opsis_session::run::StartedRun> {
    let session = session_of(&app);
    opsis_session::run::run_workflow(session.clone(), targets).await
}

#[tauri::command]
#[specta::specta]
pub fn cancel_run(app: AppHandle) {
    let session = session_of(&app);
    opsis_session::run::cancel_run(&session);
}

#[tauri::command]
#[specta::specta]
pub fn run_state(app: AppHandle) -> RunState {
    let session = session_of(&app);
    opsis_session::run::run_state(&session)
}

#[tauri::command]
#[specta::specta]
pub fn clear_run_log(app: AppHandle) {
    let session = session_of(&app);
    opsis_session::run::clear_run_log(&session);
}

#[tauri::command]
#[specta::specta]
pub fn unwatch_live(app: AppHandle, window: tauri::WebviewWindow, channel: u32) {
    let session = session_of(&app);
    opsis_session::run::unwatch_live(&session, window.label(), channel);
}

#[tauri::command]
#[specta::specta]
pub fn watch_live(
    app: AppHandle,
    window: tauri::WebviewWindow,
    frames: FrameChannel,
) -> Vec<LiveNoted> {
    let session = session_of(&app);
    let noted =
        opsis_session::run::watch_live(&session, window.label(), crate::frames::sink(frames));
    serde_json::to_value(noted)
        .and_then(serde_json::from_value)
        .unwrap_or_else(|error| {
            log::warn!("reading the live notes for a window: {error}");
            Vec::new()
        })
}

#[tauri::command]
#[specta::specta]
pub fn live_window_watching(app: AppHandle) -> bool {
    let session = session_of(&app);
    opsis_session::run::live_window_watching(&session)
}
