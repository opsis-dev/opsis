use opsis_session::error::CommandResult;
use opsis_stream::picker::{PickRequest, PickResult};
use opsis_stream::view::{ViewInput, ViewSize};
use tauri::AppHandle;

use crate::frames::FrameChannel;
use crate::session_of;

#[tauri::command]
#[specta::specta]
pub async fn pick_element(
    app: AppHandle,
    request: PickRequest,
) -> CommandResult<Option<PickResult>> {
    let session = session_of(&app);
    opsis_session::views::pick_element(session.clone(), request).await
}

#[tauri::command]
#[specta::specta]
pub fn attach_view(app: AppHandle, id: String, frames: FrameChannel) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::attach_view(session, id, crate::frames::sink(frames))
}

#[tauri::command]
#[specta::specta]
pub async fn view_input(app: AppHandle, id: String, input: ViewInput) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_input(&session, id, input).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_resize(app: AppHandle, id: String, size: ViewSize) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_resize(&session, id, size).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_navigate(app: AppHandle, id: String, url: String) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_navigate(&session, id, url).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_go(app: AppHandle, id: String, steps: i64) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_go(&session, id, steps).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_reload(app: AppHandle, id: String) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_reload(&session, id).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_switch_tab(app: AppHandle, id: String, tab: String) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_switch_tab(&session, id, tab).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_answer_dialog(
    app: AppHandle,
    id: String,
    accept: bool,
    prompt: String,
) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_answer_dialog(&session, id, accept, prompt).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_choose_files(app: AppHandle, id: String) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::views::view_choose_files(session.clone(), id).await
}

#[tauri::command]
#[specta::specta]
pub fn view_close(app: AppHandle, id: String) {
    let session = session_of(&app);
    opsis_session::views::view_close(&session, &id);
}
