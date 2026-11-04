use opsis_capture::terminal_paint::{Painted, TerminalPainter};
use opsis_compile::terminal::terminal_fonts;
use opsis_types::terminal::TerminalLook;
use tauri::{AppHandle, Manager};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use crate::engine::checked_engine_files;
use crate::state::AppState;

type Waiting<T> = Option<(Vec<u8>, T)>;

#[derive(Debug)]
pub struct PaintQueue<T> {
    latest: watch::Sender<Waiting<T>>,
    stop: CancellationToken,
}

impl<T> Drop for PaintQueue<T> {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}

impl<T: Clone + Send + Sync + 'static> PaintQueue<T> {
    pub fn start<F>(app: &AppHandle, look: TerminalLook, painted: F) -> Self
    where
        F: Fn(Painted, Vec<u8>, T) + Send + Sync + 'static,
    {
        let (latest, waiting) = watch::channel(None);
        let stop = CancellationToken::new();
        tauri::async_runtime::spawn(paint_until_stopped(
            app.clone(),
            look,
            waiting,
            stop.clone(),
            painted,
        ));
        Self { latest, stop }
    }

    pub fn offer(&self, frames: Vec<u8>, extra: T) {
        self.latest.send_replace(Some((frames, extra)));
    }
}

async fn open_painter(app: &AppHandle, stop: &CancellationToken) -> Option<TerminalPainter> {
    let state = app.state::<AppState>();
    let engine = checked_engine_files(&state, stop).await;
    let browser = match engine.browser() {
        Ok(browser) => browser.to_path_buf(),
        Err(error) => {
            log::warn!("drawing the live terminal: {error}");
            return None;
        }
    };
    let paths = &state.paths;
    match TerminalPainter::open(
        &browser,
        &paths.views_temp,
        &paths.terminal_bundle,
        &paths.public_dir,
        &terminal_fonts(),
    )
    .await
    {
        Ok(painter) => Some(painter),
        Err(error) => {
            log::warn!("opening the terminal painter: {error}");
            None
        }
    }
}

async fn paint_until_stopped<T, F>(
    app: AppHandle,
    look: TerminalLook,
    mut waiting: watch::Receiver<Waiting<T>>,
    stop: CancellationToken,
    on_painted: F,
) where
    T: Clone + Send + Sync + 'static,
    F: Fn(Painted, Vec<u8>, T) + Send + Sync + 'static,
{
    let Some(painter) = open_painter(&app, &stop).await else {
        return;
    };
    loop {
        let next = waiting.borrow_and_update().clone();
        if let Some((frames, extra)) = next {
            match painter.paint(&frames, &look).await {
                Ok((size, jpeg)) => on_painted(size, jpeg, extra),
                Err(error) => log::warn!("drawing the live terminal: {error}"),
            }
        }
        tokio::select! {
            () = stop.cancelled() => break,
            changed = waiting.changed() => if changed.is_err() { break },
        }
    }
    painter.close().await;
}
