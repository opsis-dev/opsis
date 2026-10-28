use opsis_compile::nodes::capture::{RECORD_TYPE, STILL_TYPE};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::node::{RECORDING_PORT, SESSION_PORT};
use opsis_compile::nodes::browser::{DoneWhen, Open, SignIn};
use opsis_compile::nodes::capture::{FramePicture, Record, Still, Timing};
use opsis_compile::params::Values;
use opsis_compile::recording::PREVIEW_FILE_WIDTH;
use opsis_compile::registry::node_name;
use opsis_compile::text::{format_duration, plural};
use opsis_compile::time::at_least_one_frame;
use opsis_engine::files::write_json;
use opsis_media::codecs::{H264, h264_args, strings};
use opsis_media::executors::{Tools, tool_error};
use opsis_media::ffmpeg::RunOptions;
use opsis_project::stamps::file_stamp;
use opsis_runner::context::{
    BoxFuture, ExecError, Executor, ExecutorContext, ExecutorOutput, Stamps, executor, now_ms,
};
use opsis_traces::{TakeFrame, TraceMeta, TraceTake, apply_retention, store_trace};
use opsis_types::actions::Viewport;
use opsis_types::events::{LogLevel, RunEvent};
use opsis_types::numbers;
use opsis_types::numbers::whole;
use opsis_types::secrets::{secret_file, secret_name};
use opsis_types::settings::TraceRetention;
use opsis_types::values::BrowserAction;
use opsis_types::values::{
    Artifact, ArtifactKind, BrowserSessionValue, CompileValue, ImageValue, PortValues,
    RecordingValue,
};
use tokio_util::sync::CancellationToken;

use crate::cdp::CaptureError;
use crate::encoder::{EncodeError, Encoded, FrameEncoder, FrameImage};
use crate::live::{LiveFeed, LiveSink, NodeLive};
use crate::still::{Shot, StillRequest, still_take};
use crate::still_layout::Size;
use crate::take::{
    Played, Recording, SignInError, SignIns, StepError, StepStatus, TakeEnv, record_take,
};
use crate::trace::FRAME_GAP_MS;

const RECORDING_FILE: &str = "recording.mp4";
const SIGN_IN_STAMP: &str = "secret:secretName";
const PREVIEW_FILE: &str = "recording.preview.mp4";
const ACTIONS_FILE: &str = "actions.json";
const STILL_FILE: &str = "still.png";
const IMAGE_PORT: &str = "image";
const TRACE_NOT_SAVED: &str = "Couldn't save the trace";
const PREVIEW_CRF: f64 = 26.0;
const PREVIEW_KEYFRAMES_PER_SECOND: f64 = 2.0;
const TRACE_FRAME_WIDTH: u32 = 960;
const TRACE_FRAME_QUALITY: &str = "5";

pub trait SignInStore: Send + Sync {
    fn read(&self, project_dir: &Path, name: &str) -> Result<Option<String>, SignInError>;
    fn write(&self, project_dir: &Path, name: &str, value: &str) -> Result<(), SignInError>;
}

pub struct LoginRequest {
    pub url: String,
    pub done_when: DoneWhen,
    pub selector: String,
    pub timeout_sec: u32,
    pub viewport: Option<Viewport>,
}

pub trait Login: Send + Sync {
    fn login(
        &self,
        request: LoginRequest,
        cancel: CancellationToken,
    ) -> BoxFuture<Result<String, ExecError>>;
}

pub struct CaptureDeps {
    pub tools: Tools,
    pub sign_ins: Arc<dyn SignInStore>,
    pub login: Arc<dyn Login>,
    pub live: Option<Arc<dyn LiveFeed>>,
}

struct ProjectSignIns {
    store: Arc<dyn SignInStore>,
    project_dir: PathBuf,
}

impl SignIns for ProjectSignIns {
    fn read(&self, name: &str) -> Result<Option<String>, SignInError> {
        self.store.read(&self.project_dir, name)
    }
}

fn session_input(inputs: &PortValues) -> Result<&BrowserSessionValue, ExecError> {
    match inputs.get(SESSION_PORT) {
        Some(CompileValue::BrowserSession(session)) => Ok(session),
        _ => Err(ExecError::new(
            "Connect Open browser and its actions to this node.",
        )),
    }
}

fn artifact(ctx: &ExecutorContext, kind: ArtifactKind, file: &str, label: &str) -> Artifact {
    Artifact {
        kind,
        path: ctx.rel(file),
        label: Some(label.to_owned()),
        bytes: None,
        stamp: None,
    }
}

fn take_env(ctx: &Arc<ExecutorContext>, deps: &CaptureDeps) -> Result<TakeEnv, ExecError> {
    let executable = ctx
        .env
        .engine
        .browser()
        .map_err(|e| ExecError::new(e.to_string()))?
        .to_path_buf();
    let live = deps.live.as_ref().map(|feed| {
        Arc::new(NodeLive::new(
            Arc::clone(feed),
            &ctx.run_id,
            &ctx.node.id,
            &node_name(&ctx.node),
            ctx.env.settings.live.fps,
        )) as Arc<dyn LiveSink>
    });
    let (logs, steps) = (Arc::clone(ctx), Arc::clone(ctx));
    Ok(TakeEnv {
        project_dir: ctx.project_dir.clone(),
        executable,
        temp_dir: ctx.env.temp_dir.clone(),
        live,
        trace: ctx.env.settings.traces.retention != TraceRetention::Off,
        sign_ins: Arc::new(ProjectSignIns {
            store: Arc::clone(&deps.sign_ins),
            project_dir: ctx.project_dir.clone(),
        }),
        cancel: ctx.cancel.clone(),
        log: Arc::new(move |level, message| logs.log(level, message)),
        step: Arc::new(move |node_id, status, message| {
            let run_id = steps.run_id.clone();
            let node_id = node_id.to_owned();
            (steps.emit)(match status {
                StepStatus::Start => RunEvent::NodeStart {
                    run_id,
                    node_id,
                    at: now_ms(),
                },
                StepStatus::Error => RunEvent::NodeError {
                    run_id,
                    node_id,
                    message: message.unwrap_or("Failed").to_owned(),
                    stack: None,
                    trace_id: None,
                },
                StepStatus::Done => RunEvent::NodeDone {
                    run_id,
                    node_id,
                    cached: false,
                    result: None,
                },
            });
        }),
    })
}

fn warn_failed(ctx: &ExecutorContext, what: &str, error: impl std::fmt::Display) {
    ctx.log(LogLevel::Warn, format!("{what}: {error}"));
}

async fn recording_frames(
    ctx: &ExecutorContext,
    tools: &Tools,
    video: &Path,
    origin_ms: f64,
) -> Result<Vec<TakeFrame>, String> {
    let dir = tempfile::tempdir_in(&ctx.env.temp_dir).map_err(|e| e.to_string())?;
    let args = strings(&[
        "-i",
        &video.to_string_lossy(),
        "-vf",
        &format!("fps={},scale={TRACE_FRAME_WIDTH}:-2", 1000.0 / FRAME_GAP_MS),
        "-q:v",
        TRACE_FRAME_QUALITY,
        &dir.path().join("f-%05d.jpg").to_string_lossy(),
    ]);
    tools
        .run(
            &args,
            RunOptions {
                cancel: Some(&ctx.cancel),
                duration_ms: None,
                on_progress: None,
                on_log: None,
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    let mut names: Vec<PathBuf> = std::fs::read_dir(dir.path())
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "jpg"))
        .collect();
    names.sort();
    let mut frames = Vec::new();
    for (k, file) in names.iter().enumerate() {
        let jpeg = std::fs::read(file).map_err(|e| e.to_string())?;
        let Ok(size) = imagesize::blob_size(&jpeg) else {
            continue;
        };
        frames.push(TakeFrame {
            jpeg,
            time: origin_ms + numbers::float(k) * FRAME_GAP_MS,
            width: numbers::count(size.width),
            height: numbers::count(size.height),
        });
    }
    Ok(frames)
}

async fn save_trace(
    ctx: &ExecutorContext,
    take: TraceTake,
    ok: bool,
) -> Result<Option<String>, String> {
    let (project, policy) = (ctx.project_dir.clone(), ctx.env.settings.traces.clone());
    let (node_id, label) = (ctx.node.id.clone(), node_name(&ctx.node));
    let stored = tokio::task::spawn_blocking(move || {
        let header = store_trace(
            &project,
            TraceMeta {
                node_id: &node_id,
                label: &label,
                ok,
                created_at: now_ms(),
            },
            take,
        )?;
        Ok::<_, std::io::Error>((header.id, apply_retention(&project, &policy)))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
    let (id, kept) = stored;
    match kept {
        Ok(kept) => Ok(kept.contains(&id).then_some(id)),
        Err(error) => {
            warn_failed(ctx, "Couldn't remove old traces", error);
            Ok(Some(id))
        }
    }
}

async fn finalize_trace(
    ctx: &ExecutorContext,
    tools: &Tools,
    trace: Option<TraceTake>,
    encoded: Option<(&Path, &Encoded)>,
    ok: bool,
) -> Option<String> {
    let mut trace = trace?;
    if let Some((
        video,
        Encoded {
            origin_ms: Some(origin),
            ..
        },
    )) = encoded
    {
        match recording_frames(ctx, tools, video, *origin).await {
            Ok(frames) if frames.is_empty() => ctx.log(
                LogLevel::Info,
                "Screenshots weren't added to the trace: no frames came out of the recording.",
            ),
            Ok(frames) => trace.frames.extend(frames),
            Err(error) => warn_failed(ctx, "Couldn't add screenshots to the trace", error),
        }
    }
    match save_trace(ctx, trace, ok).await {
        Ok(id) => id,
        Err(error) => {
            warn_failed(ctx, TRACE_NOT_SAVED, error);
            None
        }
    }
}

async fn finished<T>(
    ctx: &ExecutorContext,
    tools: &Tools,
    played: Played<T>,
    video: Option<&Path>,
) -> Result<(T, opsis_types::actions::ActionLog, Option<String>), ExecError> {
    let ok = played.outcome.is_ok();
    let encoded = video.zip(played.encoded.as_ref());
    let trace_id = finalize_trace(ctx, tools, played.trace, encoded, ok).await;
    match played.outcome {
        Ok(took) => Ok((took.from_page, took.log, trace_id)),
        Err(StepError::Capture(CaptureError::Cancelled)) => Err(ExecError::Cancelled),
        Err(error) if ctx.cancel.is_cancelled() => {
            log::debug!("the take stopped: {error}");
            Err(ExecError::Cancelled)
        }
        Err(error) => Err(ExecError::traced(error.to_string(), trace_id)),
    }
}

async fn preview_copy(
    ctx: &ExecutorContext,
    tools: &Tools,
    mp4: &Path,
    fps: u32,
) -> Result<Option<&'static str>, ExecError> {
    let mut args = strings(&[
        "-i",
        &mp4.to_string_lossy(),
        "-an",
        "-vf",
        &format!("scale={PREVIEW_FILE_WIDTH}:-2"),
    ]);
    args.extend(h264_args(&H264 {
        crf: PREVIEW_CRF,
        preset: Some("veryfast"),
        keyframe_every: Some(f64::from(at_least_one_frame(
            f64::from(fps) / PREVIEW_KEYFRAMES_PER_SECOND,
        ))),
        faststart: true,
    }));
    args.push(ctx.dir.join(PREVIEW_FILE).to_string_lossy().into_owned());
    match tools
        .run(
            &args,
            RunOptions {
                cancel: Some(&ctx.cancel),
                duration_ms: None,
                on_progress: None,
                on_log: None,
            },
        )
        .await
    {
        Ok(()) => Ok(Some(PREVIEW_FILE)),
        Err(_) if ctx.cancel.is_cancelled() => Err(ExecError::Cancelled),
        Err(error) => {
            warn_failed(ctx, "Couldn't make the preview copy", error);
            Ok(None)
        }
    }
}

fn encode_error(error: EncodeError) -> ExecError {
    match error {
        EncodeError::Cancelled => ExecError::Cancelled,
        EncodeError::Failed(output) => ExecError::Tool {
            message: "The video encoder couldn't start.".into(),
            output,
        },
    }
}

async fn record(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    deps: Arc<CaptureDeps>,
) -> Result<ExecutorOutput, ExecError> {
    let p: Record = ctx.params()?;
    let session = session_input(&inputs)?;
    let mp4 = ctx.dir.join(RECORDING_FILE);
    ctx.progress(0.02, Some("Starting the browser"));
    let env = take_env(&ctx, &deps)?;
    let image = if p.timing == Timing::Smooth && p.pictures == FramePicture::Lossless {
        FrameImage::Png
    } else {
        FrameImage::Jpeg
    };
    let encoder =
        FrameEncoder::start(deps.tools.as_ref(), &mp4, p.fps, image).map_err(encode_error)?;
    let fps = p.fps;
    let played = record_take(&env, session, Recording { encoder, record: p }).await;
    let frames = played.encoded.as_ref().map_or(0, |e| e.frames);
    let ((), mut log, trace_id) = finished(&ctx, &deps.tools, played, Some(&mp4)).await?;
    let probe = deps.tools.probe(&mp4).await.map_err(tool_error)?;
    if let Some(probed) = probe.duration_ms {
        log.duration_ms = log.duration_ms.min(f64::from(probed));
    }
    write_json(&ctx.dir.join(ACTIONS_FILE), &log)?;
    ctx.progress(0.95, Some("Making a preview copy"));
    let proxy = preview_copy(&ctx, &deps.tools, &mp4, fps).await?;
    ctx.log(
        LogLevel::Info,
        format!(
            "Recorded {}, {}, {}",
            format_duration(log.duration_ms),
            plural(numbers::float_wide(frames), "frame"),
            plural(numbers::float(log.actions.len()), "action")
        ),
    );
    let recording = RecordingValue {
        src: Some(ctx.project_file_ref(RECORDING_FILE)),
        proxy_src: proxy.map(|file| ctx.project_file_ref(file)),
        duration_ms: log.duration_ms,
        viewport: log.viewport,
        log: Some(log),
        node_id: ctx.node.id.clone(),
        style: Vec::new(),
    };
    Ok(ExecutorOutput {
        outputs: PortValues::from([(
            RECORDING_PORT.to_owned(),
            CompileValue::Recording(recording),
        )]),
        artifacts: vec![
            artifact(&ctx, ArtifactKind::Video, RECORDING_FILE, "Recording"),
            artifact(&ctx, ArtifactKind::Json, ACTIONS_FILE, "Action log"),
        ],
        trace_id,
        measured: Vec::new(),
    })
}

async fn still(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    deps: Arc<CaptureDeps>,
) -> Result<ExecutorOutput, ExecError> {
    let p: Still = ctx.params()?;
    let session = session_input(&inputs)?;
    let env = take_env(&ctx, &deps)?;
    let file = ctx.dir.join(STILL_FILE);
    let settings = &ctx.workflow.settings;
    let request = StillRequest {
        params: &p,
        project: Size {
            width: f64::from(settings.width),
            height: f64::from(settings.height),
        },
        file: &file,
    };
    let played = still_take(&env, session, &request).await;
    let (shot, _, trace_id): (Shot, _, _) = finished(&ctx, &deps.tools, played, None).await?;
    ctx.log(
        LogLevel::Info,
        format!(
            "Screenshot {} × {} px",
            (shot.width * shot.scale).round(),
            (shot.height * shot.scale).round()
        ),
    );
    let image = ImageValue {
        src: Some(ctx.project_file_ref(STILL_FILE)),
        width: Some(whole(shot.width)),
        height: Some(whole(shot.height)),
        scale: Some(shot.scale),
        body: shot.body,
    };
    Ok(ExecutorOutput {
        outputs: PortValues::from([(IMAGE_PORT.to_owned(), CompileValue::Image(image))]),
        artifacts: vec![artifact(
            &ctx,
            ArtifactKind::Image,
            STILL_FILE,
            "Screenshot",
        )],
        trace_id,
        measured: Vec::new(),
    })
}

async fn sign_in(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    deps: Arc<CaptureDeps>,
) -> Result<ExecutorOutput, ExecError> {
    let p: SignIn = ctx.params()?;
    let name = secret_name(&p.secret_name);
    if !ctx.forced
        && deps
            .sign_ins
            .read(&ctx.project_dir, &name)
            .map_err(|e| ExecError::new(e.to_string()))?
            .is_some()
    {
        ctx.log(LogLevel::Info, format!("Using the saved sign-in “{name}”."));
        return Ok(ExecutorOutput::default());
    }
    let viewport = match inputs.get(SESSION_PORT) {
        Some(CompileValue::BrowserSession(session)) => serde_json::from_value::<Open>(
            serde_json::Value::Object(session.browser.params.clone()),
        )
        .ok()
        .map(|o| Viewport {
            width: o.viewport_width,
            height: o.viewport_height,
        }),
        _ => None,
    };
    ctx.log(LogLevel::Info, "Sign in in the window that opened.");
    let state = deps
        .login
        .login(
            LoginRequest {
                url: p.url,
                done_when: p.done_when,
                selector: p.selector,
                timeout_sec: p.timeout_sec,
                viewport,
            },
            ctx.cancel.clone(),
        )
        .await?;
    deps.sign_ins
        .write(&ctx.project_dir, &name, &state)
        .map_err(|e| ExecError::new(e.to_string()))?;
    ctx.log(LogLevel::Info, format!("Saved the sign-in as “{name}”."));
    Ok(ExecutorOutput::default())
}

pub fn sign_in_stamp(project_dir: &Path, params: &Values) -> Stamps {
    serde_json::from_value::<SignIn>(serde_json::Value::Object(params.clone()))
        .ok()
        .and_then(|p| file_stamp(&secret_file(project_dir, &secret_name(&p.secret_name))))
        .map(|stamp| Stamps::from([(SIGN_IN_STAMP.to_owned(), stamp)]))
        .unwrap_or_default()
}

fn with_deps<F, Fut>(deps: &Arc<CaptureDeps>, run: F) -> Executor
where
    F: Fn(Arc<ExecutorContext>, PortValues, Arc<CaptureDeps>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ExecutorOutput, ExecError>> + Send + 'static,
{
    let deps = Arc::clone(deps);
    executor(move |ctx, inputs| run(ctx, inputs, Arc::clone(&deps)))
}

pub fn capture_executors(deps: &Arc<CaptureDeps>) -> Vec<(&'static str, Executor)> {
    vec![
        (RECORD_TYPE, with_deps(deps, record)),
        (STILL_TYPE, with_deps(deps, still)),
        (
            BrowserAction::SignIn.kind(),
            with_deps(deps, sign_in).stamped(sign_in_stamp),
        ),
    ]
}
