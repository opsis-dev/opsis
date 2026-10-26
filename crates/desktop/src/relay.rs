use opsis_types::shell::MenuCommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_specta::Event;

use crate::error::CommandResult;
use crate::events::{MenuCommandFired, Relayed};
use crate::windows::MAIN_WINDOW;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RelayTarget {
    Editor,
    Others,
}

#[tauri::command]
#[specta::specta]
pub fn relay(
    app: AppHandle,
    window: WebviewWindow,
    to: RelayTarget,
    message: Value,
) -> CommandResult<()> {
    let from = window.label().to_owned();
    let event = Relayed {
        from: from.clone(),
        message,
    };
    match to {
        RelayTarget::Editor => event.emit_to(&app, MAIN_WINDOW)?,
        RelayTarget::Others => {
            for label in app.webview_windows().into_keys().filter(|l| *l != from) {
                event.emit_to(&app, label.as_str())?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn send_menu_to_editor(app: AppHandle, command: MenuCommand) -> CommandResult<()> {
    if let Some(main) = app.get_webview_window(MAIN_WINDOW)
        && let Err(error) = main.set_focus()
    {
        log::warn!("raising the editor: {error}");
    }
    Ok(MenuCommandFired(command).emit_to(&app, MAIN_WINDOW)?)
}
