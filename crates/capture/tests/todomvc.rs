use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use opsis_capture::executors::{CaptureDeps, Login, LoginRequest, SignInStore, capture_executors};
use opsis_engine::paths::{ShellPlatform, engine_paths};
use opsis_media::executors::Tools;
use opsis_media::ffmpeg::Ffmpeg;
use opsis_runner::context::{BoxFuture, ExecError, Executors, RunEnv};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_types::actions::ActionLog;
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::settings::AppSettings;
use opsis_types::workflow::Workflow;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

struct NoSignIns;

impl SignInStore for NoSignIns {
    fn read(&self, _: &Path, _: &str) -> Result<Option<String>, String> {
        Ok(None)
    }

    fn write(&self, _: &Path, _: &str, _: &str) -> Result<(), String> {
        Ok(())
    }
}

impl Login for NoSignIns {
    fn login(&self, _: LoginRequest, _: CancellationToken) -> BoxFuture<Result<String, ExecError>> {
        Box::pin(async { Err(ExecError::new("No sign-in window in tests.")) })
    }
}

fn engine_dir() -> PathBuf {
    std::env::var_os("OPSIS_ENGINE_DIR").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine"),
        PathBuf::from,
    )
}

fn without_times(log: &ActionLog) -> Vec<Value> {
    log.actions
        .iter()
        .map(|a| {
            let mut value = serde_json::to_value(a).unwrap();
            for key in ["t", "tEnd", "tDepart", "tDown"] {
                value.as_object_mut().unwrap().remove(key);
            }
            value
        })
        .collect()
}

fn as_floats(value: Value) -> Value {
    match value {
        Value::Number(n) => serde_json::json!(n.as_f64().unwrap()),
        Value::Array(list) => Value::Array(list.into_iter().map(as_floats).collect()),
        Value::Object(map) => {
            Value::Object(map.into_iter().map(|(k, v)| (k, as_floats(v))).collect())
        }
        other => other,
    }
}

fn recorded_log(project: &Path) -> ActionLog {
    let cache = project.join(".opsis/cache");
    let take = std::fs::read_dir(&cache)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path().join("actions.json"))
        .find(|f| f.exists())
        .unwrap();
    serde_json::from_str(&std::fs::read_to_string(take).unwrap()).unwrap()
}

struct Ran {
    project: PathBuf,
    events: Vec<RunEvent>,
    _temp: tempfile::TempDir,
}

async fn run_example(edit: impl FnOnce(&mut Value), target: &str) -> Ran {
    let temp = tempfile::tempdir().unwrap();
    let project = std::env::var_os("OPSIS_TAKE_DIR").map_or_else(
        || temp.path().join("project"),
        |dir| PathBuf::from(dir).join(target),
    );
    std::fs::create_dir_all(&project).unwrap();
    let example =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/todomvc/project.opsis.json");
    let mut json: Value = serde_json::from_str(&std::fs::read_to_string(example).unwrap()).unwrap();
    edit(&mut json);
    let workflow: Workflow = serde_json::from_value(json).unwrap();
    let engine = engine_dir();
    let tools: Tools = Arc::new(Ffmpeg::new(&engine_paths(
        &engine,
        ShellPlatform::current().unwrap(),
    )));
    let deps = Arc::new(CaptureDeps {
        tools,
        sign_ins: Arc::new(NoSignIns),
        login: Arc::new(NoSignIns),
        live: None,
    });
    let mut executors = Executors::default();
    executors.extend(capture_executors(&deps)).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let run = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine_dir: engine,
            public_dir: project.clone(),
            temp_dir: temp.path().to_path_buf(),
            settings: AppSettings::default(),
        }),
        executors: Arc::new(executors),
        emit: Arc::new(move |e| {
            eprintln!("{e:?}");
            seen.lock().unwrap().push(e);
        }),
        cancel: CancellationToken::new(),
    };
    let request = RunRequest {
        project_dir: project.to_string_lossy().into_owned(),
        workflow,
        target_node_ids: Some(vec![target.into()]),
        from_node_id: None,
    };
    run_workflow("take", request, run).await;
    let events = std::mem::take(&mut *events.lock().unwrap());
    assert!(
        matches!(events.last(), Some(RunEvent::RunDone { ok: true, .. })),
        "{:?}",
        events.last()
    );
    Ran {
        project,
        events,
        _temp: temp,
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine and the internet"]
async fn todomvc_take_matches_the_take_recorded_before_the_move() {
    let ran = run_example(|_| {}, "record").await;
    let before: Value = serde_json::from_str(include_str!("todomvc-before.json")).unwrap();
    let log = recorded_log(&ran.project);
    assert_eq!(
        as_floats(serde_json::to_value(without_times(&log)).unwrap()),
        as_floats(before["actions"].clone())
    );
    let takes = before["takes"].as_array().unwrap();
    let frames = (log.duration_ms * 30.0 / 1000.0).round();
    let range = takes.iter().map(|t| t["frames"].as_f64().unwrap());
    let (low, high) = range
        .clone()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), f| {
            (lo.min(f), hi.max(f))
        });
    eprintln!(
        "frames {frames}, before {low}–{high}; cursor samples {}",
        log.cursor_track.as_ref().map_or(0, Vec::len)
    );
    assert_eq!(
        (log.viewport.width, log.viewport.height, log.dpr),
        (1920.0, 1080.0, 2.0)
    );
    assert!(
        frames >= low * 0.95 && frames <= high * 1.05,
        "{frames} frames against {low}–{high}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine and the internet"]
async fn takes_with_actions_shown_play_every_action() {
    let ran = run_example(
        |w| {
            w["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .filter(|n| n["id"] == "record")
                .for_each(|n| n["params"]["showActions"] = true.into())
        },
        "record",
    )
    .await;
    assert_eq!(recorded_log(&ran.project).actions.len(), 9);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine and the internet"]
async fn screenshots_one_element_with_space_around_it() {
    let ran = run_example(
        |w| {
            w["nodes"].as_array_mut().unwrap().push(serde_json::json!({ "id": "shot", "type": "capture.still", "position": { "x": 0, "y": 0 }, "params": { "region": "element", "target": "css=.todo-list", "padding": 24 } }));
            w["edges"].as_array_mut().unwrap().push(serde_json::json!({ "id": "filter.session->shot.session", "source": "filter", "sourceHandle": "session", "target": "shot", "targetHandle": "session" }));
        },
        "shot",
    )
    .await;
    let image = ran
        .events
        .iter()
        .find_map(|e| match e {
            RunEvent::NodeDone {
                node_id,
                result: Some(result),
                ..
            } if node_id == "shot" => result.outputs.get("image").cloned(),
            _ => None,
        })
        .unwrap();
    let opsis_types::values::CompileValue::Image(image) = image else {
        panic!("{image:?}")
    };
    let body = image.body.unwrap();
    eprintln!("{image:?}");
    assert_eq!((body.x, body.y), (24.0, 24.0));
    assert!(image.scale.unwrap() >= 2.0);
    let file = ran
        .project
        .join(image.src.unwrap().trim_start_matches("project:"));
    let size = imagesize::size(file).unwrap();
    assert_eq!(
        size.width as f64,
        (image.width.unwrap() * image.scale.unwrap()).round()
    );
}
