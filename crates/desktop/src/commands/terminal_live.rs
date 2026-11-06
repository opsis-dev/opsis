use opsis_session::error::CommandResult;
use opsis_types::terminal::TerminalInput;
use tauri::AppHandle;

use crate::frames::FrameChannel;
use crate::session_of;

#[tauri::command]
#[specta::specta]
pub fn attach_terminal(app: AppHandle, id: String, frames: FrameChannel) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::terminal_live::attach_terminal(&session, &id, crate::frames::sink(frames))
}

#[tauri::command]
#[specta::specta]
pub async fn terminal_input(app: AppHandle, id: String, input: TerminalInput) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::terminal_live::terminal_input(&session, id, input).await
}

#[tauri::command]
#[specta::specta]
pub fn finish_terminal(app: AppHandle, id: String, save: bool) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::terminal_live::finish_terminal(&session, &id, save)
}
