use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use opsis_compile::terminal::default_look;
use opsis_runner::context::{BoxFuture, ExecError};
use opsis_terminal::executors::{LiveTapeHost, LiveTapeRequest};
use opsis_terminal::live::{LiveOptions, LiveTape};
use opsis_terminal::tape::write;
use opsis_types::terminal::{TerminalInput, TerminalLook, TerminalTheme};
use serde::Serialize;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::Session;
use crate::error::{CommandError, CommandResult};
use crate::frames::{FrameChannel, send};
use crate::terminal_paint::PaintQueue;

const FRAME_GAP: Duration = Duration::from_millis(33);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalFrameHeader {
    skipped: u32,
    width: u32,
    height: u32,
    cols: u16,
    rows: u16,
    cell_width: f64,
    cell_height: f64,
    padding: u32,
}

type Channel = Arc<Mutex<Option<FrameChannel>>>;

#[derive(Debug)]
struct Entry {
    tape: Mutex<Option<LiveTape>>,
    theme: TerminalTheme,
    frames: Channel,
    painter: PaintQueue<u32>,
    done: Mutex<Option<oneshot::Sender<bool>>>,
    closed: CancellationToken,
}

fn painter(
    app: &Session,
    look: TerminalLook,
    cols: u16,
    rows: u16,
    frames: Channel,
) -> PaintQueue<u32> {
    let (cell_width, cell_height, padding) = (look.cell_width(), look.cell_height(), look.padding);
    PaintQueue::start(app, look, move |size, jpeg, skipped| {
        let header = TerminalFrameHeader {
            skipped,
            width: size.width,
            height: size.height,
            cols,
            rows,
            cell_width,
            cell_height,
            padding,
        };
        let mut frames = lock(&frames);
        if !frames
            .as_ref()
            .is_some_and(|channel| send(channel, &header, &jpeg))
        {
            frames.take();
        }
    })
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
        if let Some((bytes, skipped)) = frame {
            self.painter.offer(bytes, skipped);
        }
    }

    fn pump(&self) -> bool {
        lock(&self.tape)
            .as_ref()
            .is_some_and(|tape| tape.pump(Duration::ZERO))
    }
}

#[derive(Debug, Default)]
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
    app: Session,
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
    let frames: Channel = Arc::new(Mutex::new(None));
    let look = default_look(Some(request.theme.clone()));
    let entry = Arc::new(Entry {
        tape: Mutex::new(Some(tape)),
        painter: painter(&app, look, request.cols, request.rows, Arc::clone(&frames)),
        theme: request.theme,
        frames,
        done: Mutex::new(Some(sender)),
        closed: CancellationToken::new(),
    });
    lock(&app.state::<LiveTapes>().open).insert(id.clone(), Arc::clone(&entry));
    pump_until_closed(&entry);
    let closed = entry.closed.clone();
    let opened =
        app.platform()
            .open_terminal_window(&id, &request.title, Box::new(move || closed.cancel()));
    if let Err(error) = opened {
        log::error!("opening the terminal window: {error}");
        entry.closed.cancel();
    }
    let save = tokio::select! {
        answer = receiver => answer.unwrap_or(false),
        () = cancel.cancelled() => false,
        () = entry.closed.cancelled() => false,
    };
    entry.closed.cancel();
    lock(&app.state::<LiveTapes>().open).remove(&id);
    app.platform().close_terminal_window(&id);
    let tape = lock(&entry.tape).take();
    let tape = tokio::task::spawn_blocking(move || tape.map(LiveTape::finish))
        .await
        .map_err(|e| ExecError::new(e.to_string()))?;
    Ok(tape.filter(|_| save).map(|t| write(&t)))
}

#[derive(Debug)]
pub struct AppLiveTapes {
    pub app: Session,
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
pub fn attach_terminal(app: &Session, id: &str, frames: FrameChannel) -> CommandResult<()> {
    let tapes = app.state::<LiveTapes>();
    let entry = tapes.entry(id)?;
    *lock(&entry.frames) = Some(frames);
    entry.send_frame();
    Ok(())
}
pub async fn terminal_input(app: &Session, id: String, input: TerminalInput) -> CommandResult<()> {
    let tapes = app.state::<LiveTapes>();
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
pub fn finish_terminal(app: &Session, id: &str, save: bool) -> CommandResult<()> {
    let tapes = app.state::<LiveTapes>();
    let entry = tapes.entry(id)?;
    if let Some(done) = lock(&entry.done).take()
        && done.send(save).is_err()
    {
        log::debug!("the recording ended before its answer arrived");
    }
    Ok(())
}
