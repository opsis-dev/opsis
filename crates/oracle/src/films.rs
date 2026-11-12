use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use opsis_capture::executors::{CaptureDeps, Login, LoginRequest, SignInStore, capture_executors};
use opsis_capture::take::SignInError;
use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::output::RENDER_TYPE;
use opsis_engine::tools::EngineFiles;
use opsis_media::executors::{Tools, media_executors};
use opsis_media::ffmpeg::Ffmpeg;
use opsis_render::executors::{RenderEngine, output_executors};
use opsis_runner::context::{BoxFuture, ExecError, Executors, RunEnv};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_terminal::executors::{TerminalDeps, terminal_executors};
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::settings::AppSettings;
use opsis_types::workflow::Workflow;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::host::{engine_dir, film_bundle, repo};
use crate::pictures::ORACLE_GL;

pub const PROJECT_FILE: &str = "project.opsis.json";
pub const EXPORT_FOLDER: &str = ".opsis/out";
const APP_VERSION: &str = "oracle";

#[derive(Debug, thiserror::Error)]
pub enum FilmRunError {
    #[error("copying {path}: {source}")]
    Copy {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("reading the project {path}: {message}")]
    Project { path: PathBuf, message: String },
    #[error("the run failed:\n{0}")]
    Run(String),
}

struct NoSignIns;

impl SignInStore for NoSignIns {
    fn read(&self, _: &Path, _: &str) -> Result<Option<String>, SignInError> {
        Ok(None)
    }

    fn write(&self, _: &Path, _: &str, _: &str) -> Result<(), SignInError> {
        Ok(())
    }
}

impl Login for NoSignIns {
    fn login(&self, _: LoginRequest, _: CancellationToken) -> BoxFuture<Result<String, ExecError>> {
        Box::pin(async { Err(ExecError::new("The oracle has no sign-in window.")) })
    }
}

pub fn example_names() -> Result<Vec<String>, FilmRunError> {
    let dir = repo().join("examples");
    let read = std::fs::read_dir(&dir).map_err(|source| FilmRunError::Copy {
        path: dir.clone(),
        source,
    })?;
    let mut names: Vec<String> = read
        .filter_map(Result::ok)
        .filter(|e| e.path().join(PROJECT_FILE).is_file())
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .collect();
    names.sort();
    Ok(names)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), FilmRunError> {
    let failed = |path: &Path| {
        let path = path.to_path_buf();
        move |source| FilmRunError::Copy { path, source }
    };
    std::fs::create_dir_all(to).map_err(failed(to))?;
    for entry in std::fs::read_dir(from).map_err(failed(from))? {
        let entry = entry.map_err(failed(from))?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(failed(&target))?;
        }
    }
    Ok(())
}

pub fn lossless_renders(project: &mut Value) {
    let codec = serde_json::to_value(Codec::Ffv1).expect("a codec as JSON");
    let gl = serde_json::to_value(ORACLE_GL).expect("a drawing choice as JSON");
    let Some(nodes) = project.get_mut("nodes").and_then(Value::as_array_mut) else {
        return;
    };
    for node in nodes
        .iter_mut()
        .filter(|n| n.get("type").and_then(Value::as_str) == Some(RENDER_TYPE))
    {
        if !node.get("params").is_some_and(Value::is_object) {
            node["params"] = Value::Object(serde_json::Map::new());
        }
        node["params"]["codec"] = codec.clone();
        node["params"]["gl"] = gl.clone();
    }
}

pub fn prepare_example(name: &str, project: &Path) -> Result<Workflow, FilmRunError> {
    copy_tree(&repo().join("examples").join(name), project)?;
    let file = project.join(PROJECT_FILE);
    let unreadable = |message: String| FilmRunError::Project {
        path: file.clone(),
        message,
    };
    let text = std::fs::read_to_string(&file).map_err(|e| unreadable(e.to_string()))?;
    let mut json: Value = serde_json::from_str(&text).map_err(|e| unreadable(e.to_string()))?;
    lossless_renders(&mut json);
    let edited = serde_json::to_string_pretty(&json).expect("a project as JSON");
    std::fs::write(&file, edited).map_err(|e| unreadable(e.to_string()))?;
    serde_json::from_value(json).map_err(|e| unreadable(e.to_string()))
}

fn executors(tools: &Tools) -> Executors {
    let render = Arc::new(RenderEngine {
        tools: Arc::clone(tools),
        film_bundle: film_bundle(),
    });
    let capture = Arc::new(CaptureDeps {
        tools: Arc::clone(tools),
        sign_ins: Arc::new(NoSignIns),
        login: Arc::new(NoSignIns),
        live: None,
    });
    let terminal = TerminalDeps {
        live_tapes: None,
        live_frames: None,
        previews: None,
    };
    let mut all = Executors::default();
    for list in [
        media_executors(tools),
        output_executors(&render),
        capture_executors(&capture),
        terminal_executors(&terminal),
    ] {
        all.extend(list)
            .expect("executors with distinct node types");
    }
    all
}

fn failures(events: &[RunEvent]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::NodeError {
                node_id, message, ..
            } => Some(format!("{node_id}: {message}")),
            _ => None,
        })
        .collect()
}

#[derive(Debug)]
pub struct ExampleRun<'a> {
    pub project: &'a Path,
    pub temp: &'a Path,
    pub ffmpeg: &'a Path,
}

pub async fn run_example(run: &ExampleRun<'_>, workflow: Workflow) -> Result<(), FilmRunError> {
    let engine = EngineFiles {
        ffmpeg: Some(run.ffmpeg.to_path_buf()),
        ..EngineFiles::downloaded(&engine_dir())
    };
    let tools: Tools = Arc::new(Ffmpeg::new(&engine).with_temp(run.temp));
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let deps = RunDeps {
        env: Arc::new(RunEnv {
            app_version: APP_VERSION.into(),
            engine,
            public_dir: repo().join("public"),
            temp_dir: run.temp.to_path_buf(),
            settings: AppSettings::default(),
        }),
        executors: Arc::new(executors(&tools)),
        emit: Arc::new(move |e| seen.lock().expect("run events").push(e)),
        cancel: CancellationToken::new(),
    };
    let request = RunRequest {
        project_dir: run.project.to_string_lossy().into_owned(),
        workflow,
        target_node_ids: None,
        from_node_id: None,
    };
    run_workflow("oracle", request, deps).await;
    let events = std::mem::take(&mut *events.lock().expect("run events"));
    let failed = failures(&events);
    let done = matches!(events.last(), Some(RunEvent::RunDone { ok: true, .. }));
    if failed.is_empty() && done {
        Ok(())
    } else {
        Err(FilmRunError::Run(format!(
            "{}\nlast event: {:?}",
            failed.join("\n"),
            events.last()
        )))
    }
}
