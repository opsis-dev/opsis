use std::sync::{Arc, Mutex};

use opsis_engine::tools::EngineFiles;
use opsis_project::stamps::file_stamp;
use opsis_runner::context::{Executor, ExecutorOutput, Executors, RunEnv, Stamps, executor};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::secrets::secret_file;
use opsis_types::settings::AppSettings;
use opsis_types::workflow::Workflow;
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn signed_in_take() -> Workflow {
    let node = |id: &str, kind: &str, params: serde_json::Value| json!({ "id": id, "type": kind, "position": { "x": 0, "y": 0 }, "params": params });
    let edge = |a: &str, b: &str| json!({ "id": format!("{a}-{b}"), "source": a, "sourceHandle": "session", "target": b, "targetHandle": "session" });
    serde_json::from_value(json!({
        "version": 1, "id": "x", "name": "x",
        "nodes": [
            node("open", "browser.open", json!({})),
            node("login", "browser.signIn", json!({ "url": "https://x.test/login" })),
            node("go", "browser.navigate", json!({ "url": "https://x.test/app" })),
            node("rec", "capture.record", json!({}))
        ],
        "edges": [edge("open", "login"), edge("login", "go"), edge("go", "rec")]
    }))
    .unwrap()
}

struct Run {
    ran: Arc<Mutex<Vec<String>>>,
    events: Arc<Mutex<Vec<RunEvent>>>,
}

async fn run(dir: &std::path::Path, workflow: &Workflow) -> Run {
    let ran = Arc::new(Mutex::new(Vec::new()));
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut executors = Executors::default();
    for kind in ["browser.signIn", "capture.record"] {
        let ran = Arc::clone(&ran);
        let run = executor(move |ctx, _| {
            let ran = Arc::clone(&ran);
            async move {
                ran.lock().unwrap().push(ctx.node.id.clone());
                if ctx.node.kind == "browser.signIn" {
                    let file = secret_file(&ctx.project_dir, "default");
                    std::fs::create_dir_all(file.parent().unwrap())?;
                    std::fs::write(file, "sealed sign-in")?;
                }
                Ok(ExecutorOutput::default())
            }
        });
        let run = if kind == "browser.signIn" {
            run.stamped(|dir, _| {
                file_stamp(&secret_file(dir, "default"))
                    .map_or_default(|stamp| Stamps::from([("signIn".to_owned(), stamp)]))
            })
        } else {
            run
        };
        executors.add(kind, run).unwrap();
    }
    let seen = Arc::clone(&events);
    let deps = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine: EngineFiles::default(),
            public_dir: dir.into(),
            temp_dir: dir.into(),
            settings: AppSettings::default(),
        }),
        executors: Arc::new(executors),
        emit: Arc::new(move |e| seen.lock().unwrap().push(e)),
        cancel: CancellationToken::new(),
    };
    let request = RunRequest {
        project_dir: dir.to_string_lossy().into_owned(),
        workflow: workflow.clone(),
        target_node_ids: None,
        from_node_id: None,
    };
    run_workflow("run", request, deps).await;
    Run { ran, events }
}

#[tokio::test]
async fn runs_the_sign_in_first_as_the_node_before_the_take() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &signed_in_take()).await;
    assert_eq!(*out.ran.lock().unwrap(), ["login", "rec"]);
    assert!(matches!(
        out.events.lock().unwrap().last(),
        Some(RunEvent::RunDone { ok: true, .. })
    ));
}

#[tokio::test]
async fn keeps_what_it_recorded_with_a_new_sign_in_so_the_next_run_is_all_saved_results() {
    let dir = tempfile::tempdir().unwrap();
    let workflow = signed_in_take();
    run(dir.path(), &workflow).await;
    let again = run(dir.path(), &workflow).await;
    assert!(again.ran.lock().unwrap().is_empty());
    let done: Vec<(String, bool)> = again
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            RunEvent::NodeDone {
                node_id, cached, ..
            } if node_id == "login" || node_id == "rec" => Some((node_id.clone(), *cached)),
            _ => None,
        })
        .collect();
    assert_eq!(done, [("login".to_owned(), true), ("rec".to_owned(), true)]);
}

async fn run_with(dir: &std::path::Path, executors: Executors) -> Vec<RunEvent> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let deps = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine: EngineFiles::default(),
            public_dir: dir.into(),
            temp_dir: dir.into(),
            settings: AppSettings::default(),
        }),
        executors: Arc::new(executors),
        emit: Arc::new(move |e| seen.lock().unwrap().push(e)),
        cancel: CancellationToken::new(),
    };
    let request = RunRequest {
        project_dir: dir.to_string_lossy().into_owned(),
        workflow: signed_in_take(),
        target_node_ids: None,
        from_node_id: None,
    };
    run_workflow("run", request, deps).await;
    std::mem::take(&mut *events.lock().unwrap())
}

fn take_executors(sign_in: Executor) -> Executors {
    let mut executors = Executors::default();
    executors.add("browser.signIn", sign_in).unwrap();
    executors
        .add(
            "capture.record",
            executor(|_, _| async { Ok(ExecutorOutput::default()) }),
        )
        .unwrap();
    executors
}

#[tokio::test]
async fn fails_a_node_whose_executor_panics_and_still_ends_the_run() {
    let dir = tempfile::tempdir().unwrap();
    let panics = executor(|_, _| async { panic!("the keyring blocked the runtime") });
    let events = run_with(dir.path(), take_executors(panics)).await;
    let failed: Vec<(&str, &str)> = events
        .iter()
        .filter_map(|e| match e {
            RunEvent::NodeError {
                node_id, message, ..
            } => Some((node_id.as_str(), message.as_str())),
            _ => None,
        })
        .collect();
    assert_eq!(failed, [("login", "This node stopped unexpectedly.")]);
    assert!(matches!(
        events.last(),
        Some(RunEvent::RunDone { ok: false, .. })
    ));
}

#[tokio::test]
async fn reports_a_failed_node_skips_what_follows_and_names_it_in_the_run_result() {
    let dir = tempfile::tempdir().unwrap();
    let fails = executor(|_, _| async {
        Err(opsis_runner::context::ExecError::new(
            "The sign-in window closed before you signed in.",
        ))
    });
    let events = run_with(dir.path(), take_executors(fails)).await;
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::NodeSkipped { node_id, .. } if node_id == "rec"))
    );
    match events.last().unwrap() {
        RunEvent::RunDone {
            ok,
            error,
            failed_node_ids,
            skipped_node_ids,
            ..
        } => {
            assert!(!ok);
            assert_eq!(error.as_deref(), Some("Sign in failed, 1 skipped"));
            assert_eq!(
                (
                    failed_node_ids.clone().unwrap(),
                    skipped_node_ids.clone().unwrap()
                ),
                (vec!["login".to_owned()], vec!["rec".to_owned()])
            );
        }
        other => panic!("{other:?}"),
    }
    let last = opsis_project::last_run::read_last_run(dir.path())
        .unwrap()
        .unwrap();
    assert_eq!(
        last.failures.unwrap()["login"].message,
        "The sign-in window closed before you signed in."
    );
}
