use std::path::{Path, PathBuf};

use opsis_compile::terminal::terminal_fonts;
use opsis_film::TerminalPainter;
use opsis_types::terminal::TerminalLook;
use opsis_types::terminal_frames::read as read_frames;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use crate::Session;
use crate::state::AppState;

const JPEG_QUALITY: u8 = 85;
const APP_SCHEME: &str = "app:";

type Waiting<T> = Option<(Vec<u8>, T)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Painted {
    pub width: u32,
    pub height: u32,
}

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
    pub fn start<F>(app: &Session, look: TerminalLook, painted: F) -> Self
    where
        F: Fn(Painted, Vec<u8>, T) + Send + Sync + 'static,
    {
        let (latest, waiting) = watch::channel(None);
        let stop = CancellationToken::new();
        let public = app.state::<AppState>().paths.public_dir.clone();
        let stopping = stop.clone();
        crate::runtime::spawn_blocking(move || {
            paint_until_stopped(&public, &look, waiting, &stopping, &painted);
        });
        Self { latest, stop }
    }

    pub fn offer(&self, frames: Vec<u8>, extra: T) {
        self.latest.send_replace(Some((frames, extra)));
    }
}

fn app_file(public: &Path, source: &str) -> Option<PathBuf> {
    let rest = source.strip_prefix(APP_SCHEME)?.trim_start_matches('/');
    Some(public.join(rest))
}

fn jpeg(rgba: &[u8], (width, height): (u32, u32)) -> Option<Vec<u8>> {
    let rgb: Vec<u8> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[r, g, b, _]| [*r, *g, *b])
        .collect();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY)
        .encode(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some(out)
}

fn painted_frame(
    painter: &mut TerminalPainter<'_>,
    frames: &[u8],
    look: &TerminalLook,
) -> Result<(Painted, Vec<u8>), String> {
    let frames = read_frames(frames).map_err(|e| e.to_string())?;
    let surface = painter.paint(&frames, look).map_err(|e| e.to_string())?;
    let size = (surface.width(), surface.height());
    let picture = jpeg(&surface.rgba(), size).ok_or("the picture couldn't be encoded")?;
    Ok((
        Painted {
            width: size.0,
            height: size.1,
        },
        picture,
    ))
}

fn paint_until_stopped<T, F>(
    public: &Path,
    look: &TerminalLook,
    mut waiting: watch::Receiver<Waiting<T>>,
    stop: &CancellationToken,
    on_painted: &F,
) where
    T: Clone + Send + Sync + 'static,
    F: Fn(Painted, Vec<u8>, T) + Send + Sync + 'static,
{
    let files = |source: &str| std::fs::read(app_file(public, source)?).ok();
    let mut painter = match TerminalPainter::new(&files, &terminal_fonts()) {
        Ok(painter) => painter,
        Err(error) => {
            log::warn!("opening the terminal painter: {error}");
            return;
        }
    };
    let handle = crate::runtime::handle();
    loop {
        let next = waiting.borrow_and_update().clone();
        if let Some((frames, extra)) = next {
            match painted_frame(&mut painter, &frames, look) {
                Ok((size, picture)) => on_painted(size, picture, extra),
                Err(error) => log::warn!("drawing the live terminal: {error}"),
            }
        }
        let go_on = handle.block_on(async {
            tokio::select! {
                () = stop.cancelled() => false,
                changed = waiting.changed() => changed.is_ok(),
            }
        });
        if !go_on {
            break;
        }
    }
}
