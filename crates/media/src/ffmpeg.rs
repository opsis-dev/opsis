use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;

use futures::future::BoxFuture;
use opsis_engine::paths::EngineMissing;
use opsis_engine::tools::EngineFiles;
use opsis_types::numbers;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio_util::sync::CancellationToken;

const STDERR_LIMIT: usize = 200_000;
const STDERR_KEEP: usize = 100_000;
const STDERR_TAIL_LINES: usize = 6;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

static PROGRESS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"out_time_(?:us|ms)=(\d+)").expect("pattern"));

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error(transparent)]
    Missing(#[from] EngineMissing),
    #[error("Cancelled")]
    Cancelled,
    #[error("The video engine couldn't process a file.")]
    Failed { output: String },
}

impl ToolError {
    pub fn output(&self) -> Option<&str> {
        match self {
            Self::Failed { output } => Some(output),
            _ => None,
        }
    }
}

pub type OnProgress<'a> = &'a (dyn Fn(f64) + Send + Sync);
pub type OnLog<'a> = &'a (dyn Fn(&str) + Send + Sync);

#[derive(Default, Clone, Copy)]
pub struct RunOptions<'a> {
    pub cancel: Option<&'a CancellationToken>,
    pub duration_ms: Option<f64>,
    pub on_progress: Option<OnProgress<'a>>,
    pub on_log: Option<OnLog<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: Option<f64>,
    pub pix_fmt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    pub codec: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub duration_ms: Option<u32>,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioInfo>,
    #[serde(skip)]
    pub raw: Value,
}

#[derive(Default, Deserialize)]
struct Disposition {
    attached_pic: Option<u8>,
}

#[derive(Default, Deserialize)]
struct RawStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    pix_fmt: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    duration: Option<String>,
    disposition: Option<Disposition>,
}

#[derive(Default, Deserialize)]
struct RawFormat {
    duration: Option<String>,
    format_name: Option<String>,
}

#[derive(Default, Deserialize)]
struct RawProbe {
    format: Option<RawFormat>,
    streams: Option<Vec<RawStream>>,
}

fn seconds_to_ms(text: &str) -> Option<u32> {
    let seconds: f64 = text.trim().parse().ok()?;
    let ms = (seconds * 1000.0).round();
    (ms.is_finite() && ms > 0.0 && ms <= f64::from(u32::MAX))
        .then_some(numbers::truncate::<u32>(ms))
}

fn frame_rate(rate: &str) -> Option<f64> {
    let (count, per) = rate.split_once('/').unwrap_or((rate, "1"));
    let (count, per): (f64, f64) = (count.trim().parse().ok()?, per.trim().parse().ok()?);
    let fps = count / per;
    (fps.is_finite() && fps > 0.0).then_some(fps)
}

pub fn parse_probe(json: Value) -> Result<ProbeResult, ToolError> {
    let raw: RawProbe = serde_json::from_value(json.clone()).map_err(|e| ToolError::Failed {
        output: format!("ffprobe: unexpected output: {e}"),
    })?;
    let streams = raw.streams.unwrap_or_default();
    let is_picture = |s: &&RawStream| {
        s.codec_type.as_deref() == Some("video")
            && s.disposition.as_ref().and_then(|d| d.attached_pic) != Some(1)
    };
    let v = streams.iter().find(is_picture);
    let a = streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"));
    let format = raw.format.unwrap_or_default();
    let duration_ms = [
        format.duration.as_deref(),
        v.and_then(|v| v.duration.as_deref()),
        a.and_then(|a| a.duration.as_deref()),
    ]
    .into_iter()
    .flatten()
    .next()
    .and_then(seconds_to_ms);
    Ok(ProbeResult {
        duration_ms,
        format: format.format_name.unwrap_or_default(),
        video: v.map(|v| VideoInfo {
            codec: v.codec_name.clone().unwrap_or_default(),
            width: v.width.unwrap_or_default(),
            height: v.height.unwrap_or_default(),
            fps: v
                .avg_frame_rate
                .as_deref()
                .and_then(frame_rate)
                .or_else(|| v.r_frame_rate.as_deref().and_then(frame_rate)),
            pix_fmt: v.pix_fmt.clone().unwrap_or_default(),
        }),
        audio: a.map(|a| AudioInfo {
            codec: a.codec_name.clone().unwrap_or_default(),
            sample_rate: a.sample_rate.as_deref().and_then(|r| r.trim().parse().ok()),
            channels: a.channels,
        }),
        raw: json,
    })
}

pub fn can_stream_copy(a: &ProbeResult, b: &ProbeResult) -> bool {
    let (Some(av), Some(bv)) = (&a.video, &b.video) else {
        return false;
    };
    let sound = |p: &ProbeResult| p.audio.as_ref().map(|x| (x.codec.clone(), x.sample_rate));
    let same_rate = match (av.fps, bv.fps) {
        (Some(x), Some(y)) => (x - y).abs() < 0.01,
        (x, y) => x == y,
    };
    av.codec == bv.codec
        && av.width == bv.width
        && av.height == bv.height
        && av.pix_fmt == bv.pix_fmt
        && same_rate
        && sound(a) == sound(b)
}

fn progress_in(line: &str, opts: &RunOptions<'_>) {
    let (Some(on_progress), Some(duration)) =
        (opts.on_progress, opts.duration_ms.filter(|d| *d != 0.0))
    else {
        return;
    };
    for found in PROGRESS.captures_iter(line) {
        let ms = found[1].parse::<f64>().unwrap_or(0.0) / 1000.0;
        on_progress((ms / duration).clamp(0.0, 1.0));
    }
}

async fn read_lines(stream: impl AsyncRead + Unpin, mut each: impl FnMut(&str)) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        each(&line);
    }
}

fn tail_of(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.split('\n').collect();
    lines[lines.len().saturating_sub(STDERR_TAIL_LINES)..].join("\n")
}

#[derive(Clone)]
pub struct Ffmpeg {
    ffmpeg: Option<PathBuf>,
    ffprobe: Option<PathBuf>,
    env: Vec<(&'static str, PathBuf)>,
}

impl Ffmpeg {
    pub fn new(engine: &EngineFiles) -> Self {
        Self {
            ffmpeg: engine.ffmpeg.clone(),
            ffprobe: engine.ffprobe.clone(),
            env: Vec::new(),
        }
    }

    pub fn with_temp(mut self, dir: &Path) -> Self {
        self.env = opsis_engine::temp::temp_env(dir).into_iter().collect();
        self
    }

    fn command(
        &self,
        exe: Option<&Path>,
        args: &[String],
        cancel: Option<&CancellationToken>,
    ) -> Result<Command, ToolError> {
        let exe = exe.filter(|e| e.exists()).ok_or(EngineMissing)?;
        if cancel.is_some_and(CancellationToken::is_cancelled) {
            return Err(ToolError::Cancelled);
        }
        let mut command = Command::new(exe);
        command
            .args(args)
            .envs(self.env.iter().map(|(k, v)| (*k, v)))
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        Ok(command)
    }

    fn ffmpeg_args(args: &[String], opts: &RunOptions<'_>) -> Vec<String> {
        let mut all = vec!["-hide_banner".to_owned(), "-y".to_owned()];
        if opts.on_progress.is_some() {
            all.extend([
                "-progress".to_owned(),
                "pipe:1".to_owned(),
                "-nostats".to_owned(),
            ]);
        }
        all.extend(args.iter().cloned());
        all
    }

    async fn settle(
        mut child: Child,
        name: &str,
        opts: RunOptions<'_>,
        stdout: &mut (dyn FnMut(&str) + Send),
    ) -> Result<(), ToolError> {
        let out = child.stdout.take();
        let err = child.stderr.take();
        let mut stderr = String::new();
        let reading = async {
            let read_out = async {
                if let Some(out) = out {
                    read_lines(out, |line| stdout(line)).await;
                }
            };
            let read_err = async {
                if let Some(err) = err {
                    read_lines(err, |line| {
                        stderr.push_str(line);
                        stderr.push('\n');
                        if stderr.len() > STDERR_LIMIT {
                            let cut = stderr.len() - STDERR_KEEP;
                            let cut = (cut..stderr.len())
                                .find(|i| stderr.is_char_boundary(*i))
                                .unwrap_or(cut);
                            stderr.drain(..cut);
                        }
                        if let Some(on_log) = opts.on_log
                            && !line.trim().is_empty()
                        {
                            on_log(line);
                        }
                    })
                    .await;
                }
            };
            tokio::join!(read_out, read_err);
            child.wait().await
        };
        let cancelled = async {
            match opts.cancel {
                Some(cancel) => cancel.cancelled().await,
                None => std::future::pending().await,
            }
        };
        let status = tokio::select! {
            status = reading => status,
            () = cancelled => return Err(ToolError::Cancelled),
        };
        match status {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => {
                let code = status
                    .code()
                    .map_or_else(|| "a signal".to_owned(), |c| c.to_string());
                Err(ToolError::Failed {
                    output: format!("{name} exited with {code}: {}", tail_of(&stderr)),
                })
            }
            Err(error) => Err(ToolError::Failed {
                output: format!("{name}: {error}"),
            }),
        }
    }

    fn spawn(mut command: Command, name: &str) -> Result<Child, ToolError> {
        command.spawn().map_err(|e| ToolError::Failed {
            output: format!("{name}: {e}"),
        })
    }

    pub async fn run(&self, args: &[String], opts: RunOptions<'_>) -> Result<(), ToolError> {
        let mut command = self.command(
            self.ffmpeg.as_deref(),
            &Self::ffmpeg_args(args, &opts),
            opts.cancel,
        )?;
        command.stdin(Stdio::null());
        let child = Self::spawn(command, "ffmpeg")?;
        Self::settle(child, "ffmpeg", opts, &mut |line| progress_in(line, &opts)).await
    }

    pub fn pipe(
        &self,
        args: &[String],
        cancel: Option<CancellationToken>,
    ) -> Result<FramePipe, ToolError> {
        let quiet = RunOptions::default();
        let mut command = self.command(
            self.ffmpeg.as_deref(),
            &Self::ffmpeg_args(args, &quiet),
            cancel.as_ref(),
        )?;
        command.stdin(Stdio::piped());
        let mut child = Self::spawn(command, "ffmpeg")?;
        let stdin = child.stdin.take().ok_or_else(|| ToolError::Failed {
            output: "ffmpeg: no input pipe".to_owned(),
        })?;
        let draining = tokio::spawn(async move {
            let opts = RunOptions {
                cancel: cancel.as_ref(),
                ..RunOptions::default()
            };
            Self::settle(child, "ffmpeg", opts, &mut |_| {}).await
        });
        let done = Box::pin(async move {
            draining.await.map_err(|e| ToolError::Failed {
                output: format!("ffmpeg: {e}"),
            })?
        });
        Ok(FramePipe {
            stdin: Box::pin(stdin),
            done,
        })
    }

    pub async fn probe(&self, file: &Path) -> Result<ProbeResult, ToolError> {
        let args = [
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ]
        .iter()
        .map(|&a| a.to_owned())
        .chain(std::iter::once(file.to_string_lossy().into_owned()))
        .collect::<Vec<_>>();
        let mut command = self.command(self.ffprobe.as_deref(), &args, None)?;
        command.stdin(Stdio::null());
        let child = Self::spawn(command, "ffprobe")?;
        let mut out = String::new();
        Self::settle(child, "ffprobe", RunOptions::default(), &mut |line| {
            out.push_str(line);
            out.push('\n');
        })
        .await?;
        let json = serde_json::from_str(&out).map_err(|e| ToolError::Failed {
            output: format!("ffprobe: unexpected output: {e}"),
        })?;
        parse_probe(json)
    }
}

pub type ToolFuture<'a, T> = BoxFuture<'a, Result<T, ToolError>>;

pub struct FramePipe {
    pub stdin: std::pin::Pin<Box<dyn tokio::io::AsyncWrite + Send>>,
    pub done: ToolFuture<'static, ()>,
}

pub trait MediaTools: Send + Sync {
    fn run<'a>(&'a self, args: &'a [String], opts: RunOptions<'a>) -> ToolFuture<'a, ()>;
    fn probe<'a>(&'a self, file: &'a Path) -> ToolFuture<'a, ProbeResult>;
    fn pipe(
        &self,
        args: &[String],
        cancel: Option<CancellationToken>,
    ) -> Result<FramePipe, ToolError>;
}

impl MediaTools for Ffmpeg {
    fn run<'a>(&'a self, args: &'a [String], opts: RunOptions<'a>) -> ToolFuture<'a, ()> {
        Box::pin(Ffmpeg::run(self, args, opts))
    }

    fn probe<'a>(&'a self, file: &'a Path) -> ToolFuture<'a, ProbeResult> {
        Box::pin(Ffmpeg::probe(self, file))
    }

    fn pipe(
        &self,
        args: &[String],
        cancel: Option<CancellationToken>,
    ) -> Result<FramePipe, ToolError> {
        Ffmpeg::pipe(self, args, cancel)
    }
}
