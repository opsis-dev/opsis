use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;

use futures::future::BoxFuture;
use opsis_engine::paths::{EngineMissing, EnginePaths};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio_util::sync::CancellationToken;

use opsis_compile::text::js_round;

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
    pub width: f64,
    pub height: f64,
    pub fps: f64,
    pub pix_fmt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    pub codec: String,
    pub sample_rate: f64,
    pub channels: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub duration_ms: f64,
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
    attached_pic: Option<f64>,
}

#[derive(Default, Deserialize)]
struct RawStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    pix_fmt: Option<String>,
    sample_rate: Option<String>,
    channels: Option<f64>,
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

fn js_number(text: &str) -> f64 {
    let text = text.trim();
    if text.is_empty() {
        0.0
    } else {
        text.parse().unwrap_or(f64::NAN)
    }
}

fn frame_rate(rate: Option<&str>) -> f64 {
    let Some(rate) = rate else { return 0.0 };
    let mut parts = rate.split('/').map(js_number);
    let n = parts.next().unwrap_or(0.0);
    match parts.next() {
        Some(d) if d != 0.0 && !d.is_nan() => n / d,
        _ => n,
    }
}

fn or_zero(value: f64) -> f64 {
    if value.is_nan() { 0.0 } else { value }
}

pub fn parse_probe(json: Value) -> Result<ProbeResult, ToolError> {
    let raw: RawProbe = serde_json::from_value(json.clone()).map_err(|e| ToolError::Failed {
        output: format!("ffprobe: unexpected output: {e}"),
    })?;
    let streams = raw.streams.unwrap_or_default();
    let is_picture = |s: &&RawStream| {
        s.codec_type.as_deref() == Some("video")
            && s.disposition.as_ref().and_then(|d| d.attached_pic) != Some(1.0)
    };
    let v = streams.iter().find(is_picture);
    let a = streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"));
    let format = raw.format.unwrap_or_default();
    let duration_text = format
        .duration
        .as_deref()
        .or(v.and_then(|v| v.duration.as_deref()))
        .or(a.and_then(|a| a.duration.as_deref()));
    let duration = duration_text.map_or(0.0, js_number);
    Ok(ProbeResult {
        duration_ms: if duration.is_finite() {
            js_round(duration * 1000.0)
        } else {
            0.0
        },
        format: format.format_name.unwrap_or_default(),
        video: v.map(|v| {
            let avg = frame_rate(v.avg_frame_rate.as_deref());
            VideoInfo {
                codec: v.codec_name.clone().unwrap_or_default(),
                width: v.width.unwrap_or(0.0),
                height: v.height.unwrap_or(0.0),
                fps: if avg != 0.0 && !avg.is_nan() {
                    avg
                } else {
                    or_zero(frame_rate(v.r_frame_rate.as_deref()))
                },
                pix_fmt: v.pix_fmt.clone().unwrap_or_default(),
            }
        }),
        audio: a.map(|a| AudioInfo {
            codec: a.codec_name.clone().unwrap_or_default(),
            sample_rate: a.sample_rate.as_deref().map_or(0.0, js_number),
            channels: a.channels.unwrap_or(0.0),
        }),
        raw: json,
    })
}

pub fn can_stream_copy(a: &ProbeResult, b: &ProbeResult) -> bool {
    let (Some(av), Some(bv)) = (&a.video, &b.video) else {
        return false;
    };
    let codec = |p: &ProbeResult| p.audio.as_ref().map_or(String::new(), |x| x.codec.clone());
    let rate = |p: &ProbeResult| p.audio.as_ref().map_or(0.0, |x| x.sample_rate);
    av.codec == bv.codec
        && av.width == bv.width
        && av.height == bv.height
        && av.pix_fmt == bv.pix_fmt
        && (av.fps - bv.fps).abs() < 0.01
        && codec(a) == codec(b)
        && rate(a) == rate(b)
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
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    env: Vec<(&'static str, PathBuf)>,
}

impl Ffmpeg {
    pub fn new(engine: &EnginePaths) -> Self {
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
        exe: &Path,
        args: &[String],
        cancel: Option<&CancellationToken>,
    ) -> Result<Command, ToolError> {
        if !exe.exists() {
            return Err(EngineMissing.into());
        }
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
        let mut command =
            self.command(&self.ffmpeg, &Self::ffmpeg_args(args, &opts), opts.cancel)?;
        command.stdin(Stdio::null());
        let child = Self::spawn(command, "ffmpeg")?;
        Self::settle(child, "ffmpeg", opts, &mut |line| progress_in(line, &opts)).await
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
        let mut command = self.command(&self.ffprobe, &args, None)?;
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

pub trait MediaTools: Send + Sync {
    fn run<'a>(&'a self, args: &'a [String], opts: RunOptions<'a>) -> ToolFuture<'a, ()>;
    fn probe<'a>(&'a self, file: &'a Path) -> ToolFuture<'a, ProbeResult>;
}

impl MediaTools for Ffmpeg {
    fn run<'a>(&'a self, args: &'a [String], opts: RunOptions<'a>) -> ToolFuture<'a, ()> {
        Box::pin(Ffmpeg::run(self, args, opts))
    }

    fn probe<'a>(&'a self, file: &'a Path) -> ToolFuture<'a, ProbeResult> {
        Box::pin(Ffmpeg::probe(self, file))
    }
}
