use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opsis_types::shell::{CloseDecision, CloseRequest};
use serde_json::json;
use tauri::{AppHandle, CloseRequestApi, Manager, State, Window};
use tauri_specta::Event;
use tokio::sync::{Notify, oneshot};

use crate::error::CommandResult;
use crate::events::{ConfirmClose, FlushSave};
use crate::run::RunHost;
use crate::state::AppState;
use crate::windows::MAIN_WINDOW;

const SHOWN_WITHIN: Duration = Duration::from_secs(2);
const FLUSH_WITHIN: Duration = Duration::from_secs(3);
const UNLOAD_WITHIN: Duration = Duration::from_secs(3);
const RUN_BUSY: &str = "A run is in progress. Recording and exporting will stop.";
const SAVE_BUSY: &str = "Your latest changes are still being saved.";

#[derive(Default)]
pub struct Exit {
    allow: AtomicBool,
    asking: AtomicBool,
    quitting: AtomicBool,
    unsaved: AtomicBool,
    question: Mutex<Option<oneshot::Sender<CloseDecision>>>,
    shown: Notify,
    flushed: Notify,
}

fn ongoing(app: &AppHandle) -> Vec<String> {
    let exit = app.state::<Exit>();
    let mut busy = Vec::new();
    if app.state::<RunHost>().is_running() {
        busy.push(RUN_BUSY.to_owned());
    }
    busy.extend(app.state::<AppState>().tasks.labels());
    if exit.unsaved.load(Ordering::Relaxed) {
        busy.push(SAVE_BUSY.to_owned());
    }
    busy
}

async fn ask(app: &AppHandle, request: CloseRequest) -> CloseDecision {
    let exit = app.state::<Exit>();
    let (sender, receiver) = oneshot::channel();
    *exit.question.lock().expect("close question") = Some(sender);
    let shown = exit.shown.notified();
    if let Err(error) = ConfirmClose(request).emit_to(app, MAIN_WINDOW) {
        log::warn!("asking the editor about closing: {error}");
    }
    let closing = CloseDecision {
        close: true,
        dont_ask: false,
    };
    if tokio::time::timeout(SHOWN_WITHIN, shown).await.is_err() {
        log::info!("the editor didn't show the close question; closing");
        exit.question.lock().expect("close question").take();
        return closing;
    }
    let decision = receiver.await.unwrap_or(closing);
    log::info!(
        "close answered: close {}, don't ask again {}",
        decision.close,
        decision.dont_ask
    );
    decision
}

async fn flush_save(app: &AppHandle) {
    let exit = app.state::<Exit>();
    if !exit.unsaved.load(Ordering::Relaxed) {
        return;
    }
    let flushed = exit.flushed.notified();
    if let Err(error) = FlushSave.emit_to(app, MAIN_WINDOW) {
        log::warn!("asking the editor to save: {error}");
    }
    if tokio::time::timeout(FLUSH_WITHIN, flushed).await.is_err() {
        log::warn!("the editor didn't finish saving before closing");
    }
}

async fn confirm(app: &AppHandle, window: &Window) -> bool {
    let ask_first = app
        .state::<AppState>()
        .db()
        .read_settings()
        .map(|s| s.app.confirm_exit)
        .unwrap_or(true);
    let busy = ongoing(app);
    if !ask_first && busy.is_empty() {
        return true;
    }
    if let Err(error) = window.unminimize().and(window.set_focus()) {
        log::warn!("raising the window: {error}");
    }
    let decision = ask(
        app,
        CloseRequest {
            ask: ask_first,
            busy,
        },
    )
    .await;
    if !decision.close {
        return false;
    }
    if ask_first && decision.dont_ask {
        let patch = serde_json::from_value(json!({ "app": { "confirmExit": false } }))
            .expect("a settings patch");
        let patched = app.state::<AppState>().db().patch_settings(&patch);
        match patched {
            Ok((settings, _)) => crate::settings::announce(app, settings),
            Err(error) => log::warn!("remembering not to ask: {error}"),
        }
    }
    flush_save(app).await;
    app.state::<RunHost>().cancel();
    true
}

fn close_even_if_page_dies(window: Window) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(UNLOAD_WITHIN).await;
        if let Err(error) = window.destroy() {
            log::debug!("the window was already gone: {error}");
        }
    });
}

pub fn close_requested(window: &Window, api: &CloseRequestApi) {
    let app = window.app_handle().clone();
    let exit = app.state::<Exit>();
    if exit.allow.load(Ordering::Relaxed) {
        return;
    }
    api.prevent_close();
    if exit.asking.swap(true, Ordering::Relaxed) {
        log::info!("close requested while already asking");
        return;
    }
    log::info!("close requested");
    let window = window.clone();
    tauri::async_runtime::spawn(async move {
        let ok = confirm(&app, &window).await;
        let exit = app.state::<Exit>();
        exit.asking.store(false, Ordering::Relaxed);
        if !ok {
            exit.quitting.store(false, Ordering::Relaxed);
            return;
        }
        log::info!("closing the main window");
        exit.allow.store(true, Ordering::Relaxed);
        close_even_if_page_dies(window.clone());
        if let Err(error) = window.close() {
            log::warn!("closing the window: {error}");
        }
    });
}

pub fn quit(app: &AppHandle) {
    let exit = app.state::<Exit>();
    exit.quitting.store(true, Ordering::Relaxed);
    match app.get_webview_window(MAIN_WINDOW) {
        Some(window) => {
            if let Err(error) = window.close() {
                log::warn!("closing the window to quit: {error}");
            }
        }
        None => app.exit(0),
    }
}

#[tauri::command]
#[specta::specta]
pub fn report_unsaved(exit: State<'_, Exit>, unsaved: bool) {
    exit.unsaved.store(unsaved, Ordering::Relaxed);
}

#[tauri::command]
#[specta::specta]
pub fn close_shown(exit: State<'_, Exit>) {
    exit.shown.notify_waiters();
}

#[tauri::command]
#[specta::specta]
pub fn decide_close(exit: State<'_, Exit>, decision: CloseDecision) -> CommandResult<()> {
    if let Some(answer) = exit.question.lock().expect("close question").take()
        && answer.send(decision).is_err()
    {
        log::debug!("the close question was already answered");
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn save_flushed(exit: State<'_, Exit>) {
    exit.flushed.notify_waiters();
}
