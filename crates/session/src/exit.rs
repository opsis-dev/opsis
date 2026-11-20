use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opsis_types::shell::{CloseDecision, CloseRequest};
use serde_json::json;
use tokio::sync::{Notify, oneshot};

use crate::Session;
use crate::events::{ConfirmClose, Emit, FlushSave, MAIN_WINDOW};
use crate::run::RunHost;
use crate::state::AppState;

const SHOWN_WITHIN: Duration = Duration::from_secs(2);
const FLUSH_WITHIN: Duration = Duration::from_secs(3);
const RUN_BUSY: &str = "A run is in progress. Recording and exporting will stop.";
const SAVE_BUSY: &str = "Your latest changes are still being saved.";

#[derive(Debug, Default)]
pub struct Exit {
    allow: AtomicBool,
    asking: AtomicBool,
    quitting: AtomicBool,
    drafts: AtomicBool,
    question: Mutex<Option<oneshot::Sender<CloseDecision>>>,
    shown: Notify,
    flushed: Notify,
}

fn ongoing(app: &Session) -> Vec<String> {
    let exit = app.state::<Exit>();
    let mut busy = Vec::new();
    if app.state::<RunHost>().is_running() {
        busy.push(RUN_BUSY.to_owned());
    }
    busy.extend(app.state::<AppState>().tasks.labels());
    if exit.drafts.load(Ordering::Relaxed) || unsaved(app) {
        busy.push(SAVE_BUSY.to_owned());
    }
    busy
}

async fn ask(app: &Session, request: CloseRequest) -> CloseDecision {
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

fn unsaved(app: &Session) -> bool {
    app.state::<AppState>()
        .editor()
        .document
        .as_ref()
        .is_some_and(opsis_document::Document::is_dirty)
}

async fn flush_save(app: &Session) {
    if tokio::time::timeout(FLUSH_WITHIN, crate::script::stop_and_wait(app))
        .await
        .is_err()
    {
        log::warn!("the script didn't stop before closing");
    }
    let exit = app.state::<Exit>();
    if exit.drafts.load(Ordering::Relaxed) {
        let flushed = exit.flushed.notified();
        if let Err(error) = FlushSave.emit_to(app, MAIN_WINDOW) {
            log::warn!("asking the editor for the values being typed: {error}");
        }
        if tokio::time::timeout(FLUSH_WITHIN, flushed).await.is_err() {
            log::warn!("the editor didn't hand over the values being typed before closing");
        }
    }
    if let Err(error) = crate::document::save(app).await {
        log::warn!("saving before closing: {}", error.0);
    }
}

#[cfg(unix)]
pub fn close_on_signals(app: &Session) {
    use tokio::signal::unix::{SignalKind, signal};
    let app = app.clone();
    crate::runtime::spawn(async move {
        let listening = (
            signal(SignalKind::terminate()),
            signal(SignalKind::interrupt()),
            signal(SignalKind::hangup()),
        );
        let (Ok(mut terminate), Ok(mut interrupt), Ok(mut hangup)) = listening else {
            log::warn!("couldn't listen for the signals that end the app");
            return;
        };
        let name = tokio::select! {
            _ = terminate.recv() => "SIGTERM",
            _ = interrupt.recv() => "SIGINT",
            _ = hangup.recv() => "SIGHUP",
        };
        log::info!("{name}: saving and closing");
        flush_save(&app).await;
        app.state::<RunHost>().cancel();
        app.platform().exit(0);
    });
}

pub async fn confirm(app: &Session) -> bool {
    if app.platform().headless() {
        log::info!("closing with no one to ask: stopping the work and exiting");
        flush_save(app).await;
        app.state::<RunHost>().cancel();
        return true;
    }
    let ask_first = app.state::<AppState>().settings().app.confirm_exit;
    let busy = ongoing(app);
    if !ask_first && busy.is_empty() {
        return true;
    }
    app.platform().raise_editor();
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
        let patched = app.state::<AppState>().patch_settings(&patch).await;
        match patched {
            Ok(settings) => crate::settings::announce(app, settings),
            Err(error) => log::warn!("remembering not to ask: {error}"),
        }
    }
    flush_save(app).await;
    app.state::<RunHost>().cancel();
    true
}

impl Exit {
    pub fn allowed(&self) -> bool {
        self.allow.load(Ordering::Relaxed)
    }

    pub fn start_asking(&self) -> bool {
        !self.asking.swap(true, Ordering::Relaxed)
    }

    pub fn finish_asking(&self, closing: bool) {
        self.asking.store(false, Ordering::Relaxed);
        if closing {
            self.allow.store(true, Ordering::Relaxed);
        } else {
            self.quitting.store(false, Ordering::Relaxed);
        }
    }

    pub fn start_quitting(&self) {
        self.quitting.store(true, Ordering::Relaxed);
    }
}

pub fn report_drafts(app: &Session, held: bool) {
    let exit = app.state::<Exit>();
    exit.drafts.store(held, Ordering::Relaxed);
}
pub fn close_shown(app: &Session) {
    let exit = app.state::<Exit>();
    exit.shown.notify_waiters();
}
pub fn decide_close(app: &Session, decision: CloseDecision) {
    let exit = app.state::<Exit>();
    if let Some(answer) = exit.question.lock().expect("close question").take()
        && answer.send(decision).is_err()
    {
        log::debug!("the close question was already answered");
    }
}
pub fn save_flushed(app: &Session) {
    let exit = app.state::<Exit>();
    exit.flushed.notify_waiters();
}
