use std::path::Path;
use std::sync::{Arc, Mutex};

use opsis_capture::executors::{CaptureDeps, Login, LoginRequest, SignInStore, capture_executors};
use opsis_capture::take::SignInError;
use opsis_engine::tools::EngineFiles;
use opsis_media::executors::Tools;
use opsis_media::ffmpeg::Ffmpeg;
use opsis_runner::context::{BoxFuture, ExecError, Executors, RunEnv};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_store::keys::Keyring;
use opsis_store::secrets::{read_secret, write_secret};
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::settings::AppSettings;
use opsis_types::workflow::Workflow;
use serde_json::json;
use tokio_util::sync::CancellationToken;

const STATE: &str = r#"{"cookies":[],"origins":[]}"#;

struct KeyringSignIns(Arc<Keyring>);

impl SignInStore for KeyringSignIns {
    fn read(&self, project_dir: &Path, name: &str) -> Result<Option<String>, SignInError> {
        Ok(read_secret(&self.0, project_dir, name)?)
    }

    fn write(&self, project_dir: &Path, name: &str, value: &str) -> Result<(), SignInError> {
        Ok(write_secret(&self.0, project_dir, name, value)?)
    }
}

struct SignedIn;

impl Login for SignedIn {
    fn login(&self, _: LoginRequest, _: CancellationToken) -> BoxFuture<Result<String, ExecError>> {
        Box::pin(async { Ok(STATE.to_owned()) })
    }
}

fn sign_in_workflow() -> Workflow {
    serde_json::from_value(json!({
        "version": 1, "id": "x", "name": "x",
        "nodes": [
            { "id": "open", "type": "browser.open", "position": { "x": 0, "y": 0 }, "params": {} },
            { "id": "login", "type": "browser.signIn", "position": { "x": 0, "y": 0 }, "params": { "url": "https://x.test/login" } }
        ],
        "edges": [
            { "id": "open-login", "source": "open", "sourceHandle": "session", "target": "login", "targetHandle": "session" }
        ]
    }))
    .unwrap()
}

fn run_sign_in(keys: &Arc<Keyring>, project_dir: &Path) -> Vec<RunEvent> {
    let tools: Tools = Arc::new(Ffmpeg::new(&EngineFiles::default()));
    let deps = Arc::new(CaptureDeps {
        tools,
        sign_ins: Arc::new(KeyringSignIns(Arc::clone(keys))),
        login: Arc::new(SignedIn),
        live: None,
    });
    let mut executors = Executors::default();
    executors.extend(capture_executors(&deps)).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let run = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine: EngineFiles::default(),
            public_dir: project_dir.into(),
            temp_dir: project_dir.into(),
            settings: AppSettings::default(),
        }),
        executors: Arc::new(executors),
        emit: Arc::new(move |e| seen.lock().unwrap().push(e)),
        cancel: CancellationToken::new(),
    };
    let request = RunRequest {
        project_dir: project_dir.to_string_lossy().into_owned(),
        workflow: sign_in_workflow(),
        target_node_ids: None,
        from_node_id: None,
    };
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(on_a_worker(run_workflow("sign-in", request, run)));
    std::mem::take(&mut *events.lock().unwrap())
}

async fn on_a_worker(run: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(run).await.unwrap();
}

#[test]
#[ignore = "needs a Secret Service session of its own: run it under dbus-run-session with gnome-keyring-daemon unlocked"]
fn saves_a_sign_in_through_the_secret_service_from_a_run() {
    let dir = tempfile::tempdir().unwrap();
    let keys = Arc::new(Keyring::platform().unwrap());
    let events = run_sign_in(&keys, dir.path());
    assert!(
        matches!(events.last(), Some(RunEvent::RunDone { ok: true, .. })),
        "{events:#?}"
    );
    assert_eq!(
        read_secret(&keys, dir.path(), "default")
            .unwrap()
            .as_deref(),
        Some(STATE)
    );
}
