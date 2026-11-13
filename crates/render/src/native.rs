use std::path::PathBuf;
use std::sync::Arc;

use image::ExtendedColorType;
use image::codecs::jpeg::JpegEncoder;
use opsis_compile::asset_ref::{FOOTAGE_FOLDER, PREPARED_FOLDER, PreparedRef};
use opsis_compile::nodes::output::StillFormat;
use opsis_film::{Film, scaled};
use opsis_footage::footage::Footage;
use opsis_media::codecs::{FilmEncoding, film_raw_encode_args};
use opsis_paint::Picture;
use opsis_prepare::serve::{PreparedDirs, prepare};
use opsis_types::numbers;
use opsis_types::timeline::TimelineSpec;
use percent_encoding::percent_decode_str;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use crate::chunks::{Schedule, Worker, render_in_order};
use crate::frames::{FilmError, FilmRun};

pub const NATIVE_BASE: &str = "native:/";
const HELD_FRAMES: usize = 8;
const CHUNK_FRAMES: usize = 30;

#[derive(Debug)]
pub struct NativeFiles {
    pub dirs: PreparedDirs,
    pub footage: Arc<Footage>,
}

fn decoded(text: &str) -> String {
    percent_decode_str(text).decode_utf8_lossy().into_owned()
}

impl NativeFiles {
    fn path_of(&self, source: &str) -> Option<PathBuf> {
        if let Some(rest) = source.strip_prefix("app:") {
            return Some(self.dirs.public.join(decoded(rest)));
        }
        let rel = decoded(source.strip_prefix(NATIVE_BASE)?);
        (!rel.starts_with(".opsis/")).then(|| self.dirs.project.join(rel))
    }

    fn footage_at(source: &str) -> Option<(&str, i64)> {
        let rest = source
            .strip_prefix(NATIVE_BASE)?
            .strip_prefix(FOOTAGE_FOLDER)?
            .strip_prefix('/')?;
        let (id, time) = rest.rsplit_once('/')?;
        Some((id, time.parse().ok()?))
    }

    pub fn bytes(&self, source: &str) -> Option<Vec<u8>> {
        if let Some(path) = source
            .strip_prefix(NATIVE_BASE)
            .and_then(|rel| rel.strip_prefix(PREPARED_FOLDER))
            .and_then(|rel| rel.strip_prefix('/'))
        {
            let prepared = PreparedRef::from_path(&decoded(path))?;
            return prepare(&self.dirs, &prepared).ok().map(|made| made.bytes);
        }
        std::fs::read(self.path_of(source)?).ok()
    }

    pub fn frame(&self, source: &str) -> Option<Picture> {
        let (id, time_us) = Self::footage_at(source)?;
        let shown = self
            .footage
            .shown(id, time_us)
            .map_err(|e| log::warn!("showing {source}: {e}"))
            .ok()?;
        let (width, height) = (shown.image.width(), shown.image.height());
        let raw = shown.image.into_raw();
        let rgba = raw
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|[r, g, b]| [*r, *g, *b, u8::MAX])
            .collect();
        Some(Picture {
            width,
            height,
            rgba,
        })
    }

    pub fn size(&self, source: &str) -> Option<(f64, f64)> {
        if let Some(picture) = Self::footage_at(source).and_then(|_| self.frame(source)) {
            return Some((f64::from(picture.width), f64::from(picture.height)));
        }
        let probe = opsis_probe::probe(&self.path_of(source)?).ok()?;
        let video = probe.video?;
        Some((f64::from(video.width), f64::from(video.height)))
    }
}

#[derive(Debug)]
pub struct NativeRender {
    pub spec: TimelineSpec,
    pub files: Arc<NativeFiles>,
    pub encoding: FilmEncoding,
    pub every_nth_frame: u32,
    pub scale: f64,
    pub output: PathBuf,
}

struct Drawer<'a> {
    film: Result<Film<'a>, String>,
    scale: f64,
}

impl Worker for Drawer<'_> {
    type Error = String;

    fn draw(&mut self, frame: u32) -> Result<Vec<u8>, String> {
        let film = self.film.as_mut().map_err(|e| e.clone())?;
        film.scaled_frame(f64::from(frame), self.scale)
            .map(|surface| surface.rgba())
            .map_err(|e| e.to_string())
    }
}

fn workers() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZero::get)
}

fn draw_all(
    job: &NativeRender,
    frames: &[u32],
    sender: &mpsc::Sender<Vec<u8>>,
) -> Result<(), String> {
    let files = &job.files;
    let bytes = |source: &str| files.bytes(source);
    let sizes = |source: &str| files.size(source);
    let pictures = |source: &str| files.frame(source);
    let make = || Drawer {
        film: Film::new(&job.spec, &bytes, &sizes)
            .map(|film| film.with_frames(&pictures))
            .map_err(|e| e.to_string()),
        scale: job.scale,
    };
    let schedule = Schedule {
        workers: workers(),
        chunk: CHUNK_FRAMES,
        held: HELD_FRAMES.max(workers() * 2),
    };
    render_in_order(frames, schedule, &make, &mut |_, picture| {
        sender
            .blocking_send(picture)
            .map_err(|_| "The video encoder stopped.".to_owned())
    })
}

pub async fn render_native(job: NativeRender, run: &FilmRun<'_>) -> Result<(), FilmError> {
    let every = job.every_nth_frame.max(1);
    let frames: Vec<u32> = (0..job.spec.duration_in_frames)
        .step_by(usize::try_from(every).unwrap_or(1))
        .collect();
    let encoding = FilmEncoding {
        fps: f64::from(job.spec.fps) / f64::from(every),
        ..job.encoding.clone()
    };
    let size = (
        scaled(job.spec.width, job.scale),
        scaled(job.spec.height, job.scale),
    );
    let args = film_raw_encode_args(&encoding, size, &job.output.to_string_lossy());
    let stop = run.cancel.child_token();
    let pipe = run.tools.pipe(&args, Some(stop.clone()))?;
    let total = f64::from(numbers::count(frames.len()).max(1));
    let (sender, mut receiver) = mpsc::channel(HELD_FRAMES);
    let drawing = tokio::task::spawn_blocking(move || draw_all(&job, &frames, &sender));
    let mut stdin = pipe.stdin;
    let mut written = 0;
    let mut failed = None;
    while let Some(picture) = receiver.recv().await {
        if run.cancel.is_cancelled() {
            failed = Some(FilmError::Cancelled);
            break;
        }
        if let Err(error) = stdin.write_all(&picture).await {
            failed = Some(FilmError::Encode(format!(
                "The video encoder stopped: {error}"
            )));
            break;
        }
        written += 1;
        (run.progress)(f64::from(numbers::count(written)) / total);
    }
    drop(receiver);
    let drawn = drawing
        .await
        .map_err(|e| FilmError::Encode(e.to_string()))?
        .map_err(FilmError::Draw);
    if let Some(error) = failed.or_else(|| drawn.err()) {
        stop.cancel();
        drop(stdin);
        if let Err(stopped) = pipe.done.await {
            log::debug!("stopping the video encoder: {stopped}");
        }
        return Err(error);
    }
    stdin
        .flush()
        .await
        .map_err(|e| FilmError::Encode(format!("The video encoder stopped: {e}")))?;
    drop(stdin);
    pipe.done.await?;
    Ok(())
}

const STILL_QUALITY: f64 = 80.0;

#[derive(Debug)]
pub struct NativeStill {
    pub spec: TimelineSpec,
    pub files: Arc<NativeFiles>,
    pub frame: u32,
    pub scale: f64,
    pub format: StillFormat,
    pub jpeg_quality: Option<f64>,
    pub output: PathBuf,
}

fn jpeg(
    rgba: &[u8],
    (width, height): (u32, u32),
    quality: Option<f64>,
) -> Result<Vec<u8>, FilmError> {
    let rgb: Vec<u8> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[r, g, b, _]| [*r, *g, *b])
        .collect();
    let quality: u8 = numbers::truncate(quality.unwrap_or(STILL_QUALITY).round().clamp(1.0, 100.0));
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality)
        .encode(&rgb, width, height, ExtendedColorType::Rgb8)
        .map_err(|e| FilmError::Encode(e.to_string()))?;
    Ok(out)
}

fn still_bytes(job: &NativeStill) -> Result<Vec<u8>, FilmError> {
    let files = &job.files;
    let bytes = |source: &str| files.bytes(source);
    let sizes = |source: &str| files.size(source);
    let pictures = |source: &str| files.frame(source);
    let mut film = Film::new(&job.spec, &bytes, &sizes)
        .map_err(|e| FilmError::Draw(e.to_string()))?
        .with_frames(&pictures);
    let last = job.spec.duration_in_frames.saturating_sub(1);
    let surface = film
        .scaled_frame(f64::from(job.frame.min(last)), job.scale)
        .map_err(|e| FilmError::Draw(e.to_string()))?;
    match job.format {
        StillFormat::Png => Ok(surface.png()),
        StillFormat::Jpeg => jpeg(
            &surface.rgba(),
            (
                scaled(job.spec.width, job.scale),
                scaled(job.spec.height, job.scale),
            ),
            job.jpeg_quality,
        ),
    }
}

pub async fn render_native_still(job: NativeStill) -> Result<(), FilmError> {
    let output = job.output.clone();
    let picture = tokio::task::spawn_blocking(move || still_bytes(&job))
        .await
        .map_err(|e| FilmError::Encode(e.to_string()))??;
    tokio::fs::write(&output, picture)
        .await
        .map_err(|e| FilmError::Encode(e.to_string()))
}
