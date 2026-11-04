use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::asset_ref::project_ref;
use opsis_compile::node::{RECORDING_PORT, STYLE_PORT};
use opsis_compile::nodes::terminal::{
    CAST_FILE_TYPE, CastFile, EXPORT_TAPE_TYPE, ExportTape, PLAY_TAPE_TYPE, PlayTape,
    RECORD_TERMINAL_TYPE, RecordTerminal, TERMINAL_PORT, tape_file_name,
};
use opsis_compile::params::Values;
use opsis_compile::terminal::default_theme;
use opsis_compile::terminal::preview::PREVIEW_FILE;
use opsis_engine::files::atomic_write;
use opsis_project::paths::{is_inside, to_posix_rel};
use opsis_project::stamps::ref_stamps;
use opsis_runner::context::{
    BoxFuture, ExecError, Executor, ExecutorContext, ExecutorOutput, Stamps, executor,
};
use opsis_types::events::LogLevel;
use opsis_types::numbers::{count, millis, whole};
use opsis_types::terminal::{TerminalSessionValue, TerminalTheme};
use opsis_types::values::{
    Artifact, ArtifactKind, CompileValue, EntriesValue, Measured, PortValues, RecordingValue,
    TakeSource, TerminalSource,
};
use tokio_util::sync::CancellationToken;

use crate::cast::{read_path, write_v3};
use crate::import::import_cast;
use crate::live_feed::{LiveFeed, TakeLive};
use crate::take::{CAST_FILE, LOG_FILE, Take, TakeError, TakeOptions, record_take, save_take};
use crate::tape::{
    Command, TapeError, parse, parse_with_sources, session_tape, speed_entry, tape_steps, write,
};

pub const LIVE_COLS: u16 = 80;
pub const LIVE_ROWS: u16 = 24;
const TAPE_STAMP: &str = "tape";

#[derive(Debug, Clone)]
pub struct LiveTapeRequest {
    pub title: String,
    pub folder: PathBuf,
    pub temp: PathBuf,
    pub cols: u16,
    pub rows: u16,
    pub theme: TerminalTheme,
}

pub trait LiveTapeHost: Send + Sync {
    fn record(
        &self,
        request: LiveTapeRequest,
        cancel: CancellationToken,
    ) -> BoxFuture<Result<Option<String>, ExecError>>;
}

pub trait TakePreview: Send + Sync {
    fn render(
        &self,
        ctx: Arc<ExecutorContext>,
        take: RecordingValue,
    ) -> BoxFuture<Result<String, ExecError>>;
}

#[derive(Clone, Default)]
pub struct TerminalDeps {
    pub live_tapes: Option<Arc<dyn LiveTapeHost>>,
    pub live_frames: Option<Arc<dyn LiveFeed>>,
    pub previews: Option<Arc<dyn TakePreview>>,
}

impl fmt::Debug for TerminalDeps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalDeps")
            .field("live_tapes", &self.live_tapes.is_some())
            .field("live_frames", &self.live_frames.is_some())
            .field("previews", &self.previews.is_some())
            .finish()
    }
}

async fn with_preview(
    ctx: &Arc<ExecutorContext>,
    deps: &TerminalDeps,
    mut take: RecordingValue,
) -> Result<RecordingValue, ExecError> {
    let Some(previews) = &deps.previews else {
        return Ok(take);
    };
    let preview = previews.render(Arc::clone(ctx), take.clone()).await?;
    if let TakeSource::Terminal(source) = &mut take.source {
        source.preview = Some(preview);
    }
    Ok(take)
}

fn failed(error: &TakeError) -> ExecError {
    match error {
        TakeError::Driver(crate::driver::DriverError::Cancelled) => ExecError::Cancelled,
        other => ExecError::new(other.to_string()),
    }
}

struct TakeRun {
    session: TerminalSessionValue,
    name: String,
    pause_before_ms: u32,
    keep_after_ms: u32,
}

async fn run_take(
    ctx: &Arc<ExecutorContext>,
    deps: &TerminalDeps,
    run: TakeRun,
) -> Result<Take, ExecError> {
    let job = Arc::clone(ctx);
    let live = deps.live_frames.as_ref().map(|feed| {
        TakeLive::new(
            Arc::clone(feed),
            &ctx.run_id,
            &ctx.node.id,
            &run.name,
            ctx.env.settings.live.fps,
        )
    });
    let take = tokio::task::spawn_blocking(move || {
        let cancelled = || job.cancel.is_cancelled();
        let mut log = |line: String| job.log(LogLevel::Warn, line);
        let take = record_take(&mut TakeOptions {
            session: &run.session,
            project_dir: &job.project_dir,
            temp: &job.env.temp_dir,
            name: &run.name,
            pause_before_ms: run.pause_before_ms,
            keep_after_ms: run.keep_after_ms,
            cancelled: &cancelled,
            log: &mut log,
            live: live.as_ref(),
        });
        if let Some(live) = &live {
            match &take {
                Ok(_) => live.end(true, None),
                Err(error) => live.end(false, Some(error.to_string())),
            }
        }
        let take = take?;
        save_take(&job.dir, &take)?;
        Ok::<_, TakeError>(take)
    })
    .await
    .map_err(|e| ExecError::new(e.to_string()))?;
    take.map_err(|e| failed(&e))
}

fn recording(
    ctx: &ExecutorContext,
    take: &Take,
    session: &TerminalSessionValue,
    theme: Option<TerminalTheme>,
) -> RecordingValue {
    ctx.log(
        LogLevel::Info,
        format!(
            "Recorded {} s in a {} × {} terminal",
            millis(take.length) / 1000.0,
            session.cols,
            session.rows
        ),
    );
    RecordingValue {
        duration_ms: millis(take.length),
        log: Some(take.log.clone()),
        node_id: ctx.node.id.clone(),
        edits: Vec::new(),
        source: TakeSource::Terminal(TerminalSource {
            cast: Some(ctx.project_file_ref(CAST_FILE)),
            cols: session.cols,
            rows: session.rows,
            theme: theme.map(Box::new),
            preview: None,
        }),
    }
}

fn take_artifacts(ctx: &ExecutorContext, take: &RecordingValue) -> Vec<Artifact> {
    let preview = match &take.source {
        TakeSource::Terminal(source) => source.preview.as_ref(),
        TakeSource::Browser(_) => None,
    };
    let mut artifacts = vec![
        ctx.artifact(ArtifactKind::File, CAST_FILE, "Terminal take"),
        ctx.artifact(ArtifactKind::Json, LOG_FILE, "Action log"),
    ];
    if preview.is_some() {
        artifacts.push(ctx.artifact(ArtifactKind::Video, PREVIEW_FILE, "Preview"));
    }
    artifacts
}

async fn record(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    deps: TerminalDeps,
) -> Result<ExecutorOutput, ExecError> {
    let p: RecordTerminal = ctx.params()?;
    let Some(CompileValue::TerminalSession(session)) = inputs.get(TERMINAL_PORT).cloned() else {
        return Err(ExecError::new("Connect an Open terminal and its actions."));
    };
    let take = run_take(
        &ctx,
        &deps,
        TakeRun {
            session: session.clone(),
            name: p.name,
            pause_before_ms: p.pause_before_ms,
            keep_after_ms: p.keep_after_ms,
        },
    )
    .await?;
    let recording = with_preview(&ctx, &deps, recording(&ctx, &take, &session, None)).await?;
    Ok(ExecutorOutput {
        artifacts: take_artifacts(&ctx, &recording),
        outputs: PortValues::from([(
            RECORDING_PORT.to_owned(),
            CompileValue::Recording(recording),
        )]),
        trace_id: None,
        measured: Vec::new(),
    })
}

fn tape_problems(errors: &[TapeError]) -> ExecError {
    let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
    ExecError::new(format!("The tape file has mistakes:\n{}", lines.join("\n")))
}

fn sourced_path(project_dir: &Path, tape_dir: &Path, path: &str) -> Result<PathBuf, String> {
    let file = tape_dir.join(path);
    if !is_inside(project_dir, &file) {
        return Err("Choose a tape inside the project folder".to_owned());
    }
    Ok(file)
}

fn read_sourced(project_dir: &Path, tape_dir: &Path, path: &str) -> Result<String, String> {
    let file = sourced_path(project_dir, tape_dir, path)?;
    std::fs::read_to_string(&file).map_err(|_| "Not in the project folder".to_owned())
}

pub fn tape_stamp(project_dir: &Path, params: &Values) -> Stamps {
    let Ok(p) = serde_json::from_value::<PlayTape>(serde_json::Value::Object(params.clone()))
    else {
        return Stamps::new();
    };
    let Some(rel) = opsis_compile::asset_ref::project_rel_path(p.file.trim()) else {
        return Stamps::new();
    };
    let tape = project_dir.join(&rel);
    let Some(tape_dir) = tape.parent() else {
        return Stamps::new();
    };
    let Some(Ok(parsed)) = std::fs::read_to_string(&tape).ok().map(|text| parse(&text)) else {
        return Stamps::new();
    };
    let refs: Vec<String> = parsed
        .commands
        .iter()
        .filter_map(|c| match c {
            Command::Source { path } => sourced_path(project_dir, tape_dir, path).ok(),
            _ => None,
        })
        .filter_map(|file| to_posix_rel(project_dir, &file))
        .map(|rel| project_ref(&rel))
        .collect();
    ref_stamps(project_dir, &refs)
        .into_iter()
        .map(|(file, stamp)| (format!("{TAPE_STAMP}:{file}"), stamp))
        .collect()
}

async fn record_live(
    ctx: &ExecutorContext,
    deps: &TerminalDeps,
    file: &Path,
    title: &str,
) -> Result<String, ExecError> {
    let Some(live) = &deps.live_tapes else {
        return Err(ExecError::new("The tape file isn't in the project folder."));
    };
    ctx.log(LogLevel::Info, "Type the tape in the window that opened.");
    let request = LiveTapeRequest {
        title: title.to_owned(),
        folder: ctx.project_dir.clone(),
        temp: ctx.env.temp_dir.clone(),
        cols: LIVE_COLS,
        rows: LIVE_ROWS,
        theme: default_theme(),
    };
    let Some(text) = live.record(request, ctx.cancel.clone()).await? else {
        return Err(ExecError::Cancelled);
    };
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    atomic_write(file, text.as_bytes())?;
    ctx.log(
        LogLevel::Info,
        format!("Saved the tape as {}", file.display()),
    );
    Ok(text)
}

async fn play_tape(
    ctx: Arc<ExecutorContext>,
    _inputs: PortValues,
    deps: TerminalDeps,
) -> Result<ExecutorOutput, ExecError> {
    let p: PlayTape = ctx.params()?;
    let file = ctx.ref_to_path(p.file.trim())?;
    let title = file
        .file_name()
        .map_or_else(|| "Tape".to_owned(), |n| n.to_string_lossy().into_owned());
    let text = if p.record_live && (ctx.forced || !file.is_file()) {
        record_live(&ctx, &deps, &file, &title).await?
    } else {
        std::fs::read_to_string(&file)
            .map_err(|_| ExecError::new("The tape file isn't in the project folder."))?
    };
    let tape_dir = file.parent().map_or_default(Path::to_path_buf);
    let project_dir = ctx.project_dir.clone();
    let read = move |path: &str| read_sourced(&project_dir, &tape_dir, path);
    let sourced = parse_with_sources(&text, &read).map_err(|e| tape_problems(&e))?;
    let mapped = tape_steps(&sourced.tape, &ctx.node.id);
    for ignored in &mapped.ignored {
        ctx.log(LogLevel::Info, format!("Not used: {ignored}"));
    }
    for warning in &mapped.warnings {
        ctx.log(LogLevel::Warn, warning.clone());
    }
    let take = run_take(
        &ctx,
        &deps,
        TakeRun {
            session: mapped.session.clone(),
            name: title,
            pause_before_ms: p.pause_before_ms,
            keep_after_ms: p.keep_after_ms,
        },
    )
    .await?;
    let recording = recording(&ctx, &take, &mapped.session, mapped.theme.clone());
    let mut recording = with_preview(&ctx, &deps, recording).await?;
    let artifacts = take_artifacts(&ctx, &recording);
    if let Some(speed) = mapped.speed {
        recording.edits.push(speed_entry(
            &ctx.node.id,
            speed,
            whole(recording.duration_ms),
        ));
    }
    Ok(ExecutorOutput {
        outputs: PortValues::from([
            (
                RECORDING_PORT.to_owned(),
                CompileValue::Recording(recording),
            ),
            (
                STYLE_PORT.to_owned(),
                CompileValue::RecordingStyle(EntriesValue {
                    entries: mapped.style.clone(),
                }),
            ),
        ]),
        artifacts,
        trace_id: None,
        measured: vec![Measured::Tape {
            steps: count(mapped.session.steps.len()),
            ignored: mapped.ignored,
            warnings: mapped.warnings,
        }],
    })
}

async fn cast_file(
    ctx: Arc<ExecutorContext>,
    _inputs: PortValues,
    deps: TerminalDeps,
) -> Result<ExecutorOutput, ExecError> {
    let p: CastFile = ctx.params()?;
    let file = ctx.ref_to_path(p.file.trim())?;
    let job = Arc::clone(&ctx);
    let limit = p
        .pause_limit_ms
        .map(|ms| std::time::Duration::from_millis(u64::from(ms)));
    let (imported, events) = tokio::task::spawn_blocking(move || {
        let cast = read_path(&file)
            .map_err(|e| ExecError::new(format!("The cast file can't be read: {e}")))?;
        let events = count(cast.events.len());
        let imported = import_cast(cast, limit).map_err(|e| ExecError::new(e.to_string()))?;
        std::fs::create_dir_all(&job.dir)?;
        atomic_write(
            &job.dir.join(CAST_FILE),
            write_v3(&imported.header, &imported.events).as_bytes(),
        )?;
        opsis_engine::files::write_json(&job.dir.join(LOG_FILE), &imported.log)?;
        Ok::<_, ExecError>((imported, events))
    })
    .await
    .map_err(|e| ExecError::new(e.to_string()))??;
    ctx.log(
        LogLevel::Info,
        format!(
            "Read {events} events: {} s in a {} × {} terminal",
            millis(imported.length) / 1000.0,
            imported.header.term_cols,
            imported.header.term_rows
        ),
    );
    let take = RecordingValue {
        duration_ms: millis(imported.length),
        log: Some(imported.log),
        node_id: ctx.node.id.clone(),
        edits: Vec::new(),
        source: TakeSource::Terminal(TerminalSource {
            cast: Some(ctx.project_file_ref(CAST_FILE)),
            cols: imported.header.term_cols,
            rows: imported.header.term_rows,
            theme: imported.theme.map(Box::new),
            preview: None,
        }),
    };
    let recording = with_preview(&ctx, &deps, take).await?;
    Ok(ExecutorOutput {
        artifacts: take_artifacts(&ctx, &recording),
        outputs: PortValues::from([(
            RECORDING_PORT.to_owned(),
            CompileValue::Recording(recording),
        )]),
        trace_id: None,
        measured: vec![Measured::Cast { events }],
    })
}

async fn export_tape(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: ExportTape = ctx.params()?;
    let Some(CompileValue::TerminalSession(session)) = inputs.get(TERMINAL_PORT) else {
        return Err(ExecError::new("Connect an Open terminal and its actions."));
    };
    let written = session_tape(session);
    for skipped in &written.skipped {
        ctx.log(
            LogLevel::Warn,
            format!("“{skipped}” has no tape command, so it was left out"),
        );
    }
    let out = ctx.out_dir.join(tape_file_name(&p.file_name));
    std::fs::create_dir_all(&ctx.out_dir)?;
    atomic_write(&out, write(&written.tape).as_bytes())?;
    let rel = to_posix_rel(&ctx.project_dir, &out)
        .ok_or_else(|| ExecError::new(format!("{} isn't in the project.", out.display())))?;
    Ok(ExecutorOutput {
        outputs: PortValues::new(),
        artifacts: vec![Artifact::labelled(ArtifactKind::File, rel, "Tape")],
        trace_id: None,
        measured: Vec::new(),
    })
}

pub fn terminal_executors(deps: &TerminalDeps) -> Vec<(&'static str, Executor)> {
    let (recorder, player, importer) = (deps.clone(), deps.clone(), deps.clone());
    vec![
        (
            RECORD_TERMINAL_TYPE,
            executor(move |ctx, inputs| record(ctx, inputs, recorder.clone())),
        ),
        (
            PLAY_TAPE_TYPE,
            executor(move |ctx, inputs| play_tape(ctx, inputs, player.clone())).stamped(tape_stamp),
        ),
        (EXPORT_TAPE_TYPE, executor(export_tape)),
        (
            CAST_FILE_TYPE,
            executor(move |ctx, inputs| cast_file(ctx, inputs, importer.clone())),
        ),
    ]
}
