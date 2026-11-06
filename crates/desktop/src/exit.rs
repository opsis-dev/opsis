use std::time::Duration;

use opsis_session::events::MAIN_WINDOW;
use opsis_session::exit::{Exit, confirm};
use tauri::{AppHandle, CloseRequestApi, Manager, Window};

use crate::session_of;

const UNLOAD_WITHIN: Duration = Duration::from_secs(3);

fn close_even_if_page_dies(window: Window) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(UNLOAD_WITHIN).await;
        if let Err(error) = window.destroy() {
            log::debug!("the window was already gone: {error}");
        }
    });
}

pub fn close_requested(window: &Window, api: &CloseRequestApi) {
    let session = session_of(window.app_handle());
    let exit = session.state::<Exit>();
    if exit.allowed() {
        return;
    }
    api.prevent_close();
    if !exit.start_asking() {
        log::info!("close requested while already asking");
        return;
    }
    log::info!("close requested");
    let window = window.clone();
    tauri::async_runtime::spawn(async move {
        let ok = confirm(&session).await;
        session.state::<Exit>().finish_asking(ok);
        if !ok {
            return;
        }
        log::info!("closing the main window");
        close_even_if_page_dies(window.clone());
        if let Err(error) = window.close() {
            log::warn!("closing the window: {error}");
        }
    });
}

pub fn quit(app: &AppHandle) {
    session_of(app).state::<Exit>().start_quitting();
    match app.get_webview_window(MAIN_WINDOW) {
        Some(window) => {
            if let Err(error) = window.close() {
                log::warn!("closing the window to quit: {error}");
            }
        }
        None => app.exit(0),
    }
}
