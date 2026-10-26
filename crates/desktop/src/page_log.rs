use serde::Deserialize;
use specta::Type;
use tauri::WebviewWindow;

const PAGE_TARGET: &str = "page";

#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum PageLogLevel {
    Error,
    Warn,
}

#[tauri::command]
#[specta::specta]
pub fn page_log(window: WebviewWindow, level: PageLogLevel, message: String) {
    let label = window.label();
    match level {
        PageLogLevel::Error => log::error!(target: PAGE_TARGET, "[{label}] {message}"),
        PageLogLevel::Warn => log::warn!(target: PAGE_TARGET, "[{label}] {message}"),
    }
}
