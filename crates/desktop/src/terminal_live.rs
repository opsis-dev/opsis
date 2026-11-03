use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use opsis_runner::context::{BoxFuture, ExecError};
use opsis_terminal::executors::{LiveTapeHost, LiveTapeRequest};
use opsis_terminal::live::{LiveOptions, LiveTape};
use opsis_terminal::tape::write;
use opsis_types::terminal::{TerminalInput, TerminalTheme};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::error::{CommandError, CommandResult};
use crate::frames::{FrameChannel, send};
use crate::windows::open_terminal_window;

const FRAME_GAP: Duration = Duration::from_millis(33);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalFrameHeader {
    skipped: u32,
}

struct Entry {
    tape: Mutex<Option<LiveTape>>,
    theme: TerminalTheme,
    frames: Mutex<Option<FrameChannel>>,
    done: Mutex<Option<oneshot::Sender<bool>>>,
    closed: CancellationToken,
}

fn lock<T>(shared: &Mutex<T>) -> MutexGuard<'_, T> {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Entry {
    fn send_frame(&self) {
        let frame = {
            let tape = lock(&self.tape);
            let Some(tape) = tape.as_ref() else {
                return;
            };
            match tape.frame(&self.theme) {
                Ok(bytes) => Some((bytes, tape.skipped())),
                Err(error) => {
                    log::warn!("drawing the live terminal: {error}");
                    None
                }
            }
        };
        let Some((bytes, skipped)) = frame else {
            return;
        };
        let mut frames = lock(&self.frames);
        let sent = frames
            .as_ref()
            .is_some_and(|channel| send(channel, &TerminalFrameHeader { skipped }, &bytes));
        if !sent {
            frames.take();
        }
    }

    fn pump(&self) -> bool {
        lock(&self.tape)
            .as_ref()
            .is_some_and(|tape| tape.pump(Duration::ZERO))
    }
}

#[derive(Default)]
pub struct LiveTapes {
    open: Mutex<HashMap<String, Arc<Entry>>>,
}

impl LiveTapes {
    fn entry(&self, id: &str) -> CommandResult<Arc<Entry>> {
        lock(&self.open)
            .get(id)
            .cloned()
            .ok_or_else(|| CommandError("That terminal is closed.".into()))
    }
}

fn pump_until_closed(entry: &Arc<Entry>) {
    let entry = Arc::clone(entry);
    std::thread::spawn(move || {
        while !entry.closed.is_cancelled() {
            if entry.pump() {
                entry.send_frame();
            }
            std::thread::sleep(FRAME_GAP);
        }
    });
}

async fn record(
    app: AppHandle,
    request: LiveTapeRequest,
    cancel: CancellationToken,
) -> Result<Option<String>, ExecError> {
    let options = request.clone();
    let tape = tokio::task::spawn_blocking(move || {
        LiveTape::start(&LiveOptions {
            folder: &options.folder,
            temp: &options.temp,
            cols: options.cols,
            rows: options.rows,
        })
    })
    .await
    .map_err(|e| ExecError::new(e.to_string()))?
    .map_err(|e| ExecError::new(e.to_string()))?;
    let id = uuid::Uuid::new_v4().simple().to_string();
    let (sender, receiver) = oneshot::channel();
    let entry = Arc::new(Entry {
        tape: Mutex::new(Some(tape)),
        theme: request.theme,
        frames: Mutex::new(None),
        done: Mutex::new(Some(sender)),
        closed: CancellationToken::new(),
    });
    lock(&app.state::<LiveTapes>().open).insert(id.clone(), Arc::clone(&entry));
    pump_until_closed(&entry);
    match open_terminal_window(&app, &id, &request.title).await {
        Ok(window) => {
            let closed = entry.closed.clone();
            window.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::Destroyed) {
                    closed.cancel();
                }
            });
        }
        Err(error) => {
            log::error!("opening the terminal window: {}", error.0);
            entry.closed.cancel();
        }
    }
    let save = tokio::select! {
        answer = receiver => answer.unwrap_or(false),
        () = cancel.cancelled() => false,
        () = entry.closed.cancelled() => false,
    };
    entry.closed.cancel();
    lock(&app.state::<LiveTapes>().open).remove(&id);
    if let Some(window) = app.get_webview_window(&crate::windows::terminal_label(&id))
        && let Err(error) = window.destroy()
    {
        log::warn!("closing the terminal window: {error}");
    }
    let tape = lock(&entry.tape).take();
    let tape = tokio::task::spawn_blocking(move || tape.map(LiveTape::finish))
        .await
        .map_err(|e| ExecError::new(e.to_string()))?;
    Ok(tape.filter(|_| save).map(|t| write(&t)))
}

pub struct AppLiveTapes {
    pub app: AppHandle,
}

impl LiveTapeHost for AppLiveTapes {
    fn record(
        &self,
        request: LiveTapeRequest,
        cancel: CancellationToken,
    ) -> BoxFuture<Result<Option<String>, ExecError>> {
        Box::pin(record(self.app.clone(), request, cancel))
    }
}

#[tauri::command]
#[specta::specta]
pub fn attach_terminal(
    tapes: State<'_, LiveTapes>,
    id: String,
    frames: FrameChannel,
) -> CommandResult<()> {
    let entry = tapes.entry(&id)?;
    *lock(&entry.frames) = Some(frames);
    entry.send_frame();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn terminal_input(
    tapes: State<'_, LiveTapes>,
    id: String,
    input: TerminalInput,
) -> CommandResult<()> {
    let entry = tapes.entry(&id)?;
    tokio::task::spawn_blocking(move || {
        let mut tape = lock(&entry.tape);
        match tape.as_mut() {
            Some(tape) => tape.input(&input).map_err(CommandError::from),
            None => Err(CommandError("That terminal is closed.".into())),
        }
    })
    .await?
}

#[tauri::command]
#[specta::specta]
pub fn finish_terminal(tapes: State<'_, LiveTapes>, id: String, save: bool) -> CommandResult<()> {
    let entry = tapes.entry(&id)?;
    if let Some(done) = lock(&entry.done).take()
        && done.send(save).is_err()
    {
        log::debug!("the recording ended before its answer arrived");
    }
    Ok(())
}
