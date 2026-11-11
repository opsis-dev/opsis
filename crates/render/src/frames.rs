use std::fmt;
use std::path::Path;

use opsis_capture::cdp::CaptureError;
use opsis_capture::film::{DriverOptions, FilmPage, FrameDriver, FrameFormat, draws_webgl};
use opsis_compile::nodes::output::{Gl, StillFormat};
use opsis_media::codecs::{FilmEncoding, film_encode_args};
use opsis_media::ffmpeg::{FramePipe, MediaTools, ToolError};
use opsis_types::numbers;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

const FRAME_QUALITY: u8 = 80;
const STILL_QUALITY: f64 = 80.0;

#[derive(Debug, thiserror::Error)]
pub enum FilmError {
    #[error("{0}")]
    Capture(#[from] CaptureError),
    #[error("{0}")]
    Encode(String),
    #[error("Cancelled")]
    Cancelled,
    #[error(
        "This computer can't draw shader and 3D effects with “{}”. Set Draw with to {}.",
        .0.label(),
        instead_of(*.0)
    )]
    NoWebGl(Gl),
}

const fn instead_of(gl: Gl) -> &'static str {
    match gl {
        Gl::Vulkan => "Graphics card or Software",
        Gl::Angle | Gl::Swangle | Gl::Swiftshader => "Software",
    }
}

async fn explained<T>(
    result: Result<T, FilmError>,
    driver: &DriverOptions<'_>,
) -> Result<T, FilmError> {
    let Err(FilmError::Capture(error)) = result else {
        return result;
    };
    if driver.gl.draws_on_graphics_card()
        && !draws_webgl(driver.executable, driver.temp_dir, driver.gl).await
    {
        log::warn!(
            "the render failed ({error}) and this browser can't draw WebGL with {:?}",
            driver.gl
        );
        return Err(FilmError::NoWebGl(driver.gl));
    }
    Err(FilmError::Capture(error))
}

impl From<ToolError> for FilmError {
    fn from(error: ToolError) -> Self {
        match error {
            ToolError::Cancelled => Self::Cancelled,
            ToolError::Failed { output } => Self::Encode(output),
            other @ ToolError::Missing(_) => Self::Encode(other.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct FilmRender<'a> {
    pub driver: DriverOptions<'a>,
    pub page: FilmPage<'a>,
    pub encoding: FilmEncoding,
    pub every_nth_frame: u32,
    pub output: &'a Path,
}

pub struct FilmRun<'a> {
    pub tools: &'a dyn MediaTools,
    pub cancel: &'a CancellationToken,
    pub progress: &'a (dyn Fn(f64) + Send + Sync),
    pub note: &'a (dyn Fn(String) + Send + Sync),
}

impl fmt::Debug for FilmRun<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilmRun")
            .field("cancel", &self.cancel)
            .finish_non_exhaustive()
    }
}

fn note_capture(driver: &FrameDriver, run: &FilmRun<'_>) {
    match driver.fell_back() {
        Some(reason) => (run.note)(format!(
            "The film was drawn with {} because drawing {} didn't work here: {reason}",
            driver.capture().label(),
            opsis_capture::film::Capture::BeginFrame.label()
        )),
        None => (run.note)(format!("The film was drawn {}.", driver.capture().label())),
    }
}

fn encoder_stopped(run: &FilmRun<'_>, error: &std::io::Error) -> FilmError {
    if run.cancel.is_cancelled() {
        FilmError::Cancelled
    } else {
        FilmError::Encode(format!("The video encoder stopped: {error}"))
    }
}

async fn write_frames(
    driver: &mut FrameDriver,
    stdin: &mut FramePipe,
    frames: &[u32],
    format: FrameFormat,
    run: &FilmRun<'_>,
) -> Result<(), FilmError> {
    let total = f64::from(numbers::count(frames.len()).max(1));
    for (done, &frame) in frames.iter().enumerate() {
        if run.cancel.is_cancelled() {
            return Err(FilmError::Cancelled);
        }
        let picture = driver.frame(frame, format).await?;
        stdin
            .stdin
            .write_all(&picture)
            .await
            .map_err(|e| encoder_stopped(run, &e))?;
        (run.progress)(f64::from(numbers::count(done + 1)) / total);
    }
    Ok(())
}

async fn encode(
    driver: &mut FrameDriver,
    job: &FilmRender<'_>,
    run: &FilmRun<'_>,
) -> Result<(), FilmError> {
    let film = driver.film();
    let every = job.every_nth_frame.max(1);
    let frames: Vec<u32> = (0..film.duration_in_frames)
        .step_by(usize::try_from(every).unwrap_or(1))
        .collect();
    let encoding = FilmEncoding {
        fps: f64::from(film.fps) / f64::from(every),
        ..job.encoding.clone()
    };
    let args = film_encode_args(&encoding, &job.output.to_string_lossy());
    let stop = run.cancel.child_token();
    let mut pipe = run.tools.pipe(&args, Some(stop.clone()))?;
    let format = if encoding.png_frames {
        FrameFormat::Png
    } else {
        FrameFormat::Jpeg(FRAME_QUALITY)
    };
    let written = write_frames(driver, &mut pipe, &frames, format, run).await;
    let FramePipe { mut stdin, done } = pipe;
    if let Err(error) = written {
        stop.cancel();
        drop(stdin);
        if let Err(stopped) = done.await {
            log::debug!("stopping the video encoder: {stopped}");
        }
        return Err(error);
    }
    stdin.flush().await.map_err(|e| encoder_stopped(run, &e))?;
    drop(stdin);
    done.await?;
    Ok(())
}

pub async fn render_film(job: &FilmRender<'_>, run: &FilmRun<'_>) -> Result<(), FilmError> {
    explained(film_once(job, run).await, &job.driver).await
}

async fn film_once(job: &FilmRender<'_>, run: &FilmRun<'_>) -> Result<(), FilmError> {
    let mut driver = FrameDriver::open(&job.driver, &job.page).await?;
    note_capture(&driver, run);
    let encoded = encode(&mut driver, job, run).await;
    driver.close().await;
    encoded
}

#[derive(Debug)]
pub struct StillRender<'a> {
    pub driver: DriverOptions<'a>,
    pub page: FilmPage<'a>,
    pub frame: u32,
    pub format: StillFormat,
    pub jpeg_quality: Option<f64>,
    pub output: &'a Path,
}

fn still_format(format: StillFormat, quality: Option<f64>) -> FrameFormat {
    match format {
        StillFormat::Png => FrameFormat::Png,
        StillFormat::Jpeg => FrameFormat::Jpeg(numbers::truncate(
            quality.unwrap_or(STILL_QUALITY).round().clamp(0.0, 100.0),
        )),
    }
}

pub async fn render_still(job: &StillRender<'_>) -> Result<(), FilmError> {
    explained(still_once(job).await, &job.driver).await
}

async fn still_once(job: &StillRender<'_>) -> Result<(), FilmError> {
    let mut driver = FrameDriver::open(&job.driver, &job.page).await?;
    let last = driver.film().duration_in_frames.saturating_sub(1);
    let picture = driver
        .frame(
            job.frame.min(last),
            still_format(job.format, job.jpeg_quality),
        )
        .await;
    driver.close().await;
    tokio::fs::write(job.output, picture?)
        .await
        .map_err(|e| FilmError::Encode(e.to_string()))
}
