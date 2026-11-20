use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use opsis_folders::HostFolders;
use opsis_persistence::keys::KeyError;
use opsis_session::Session;
use opsis_session::events::SessionEvent;
use opsis_session::paths::AppPaths;
use opsis_session::platform::{Closed, FileFilter, PickedFiles, PickedFolder, Platform, Versions};
use opsis_session::script::{ScriptSource, run_script};
use opsis_session::state::AppState;
use serde_json::{Value, json};
use tokio::runtime::Runtime;
use tokio::sync::broadcast::Receiver;

const EXAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/todomvc");
const PROJECT_FILE: &str = "project.opsis.json";
const WITHIN: Duration = Duration::from_secs(10);

fn runtime() -> &'static Runtime {
    static SHARED: OnceLock<Runtime> = OnceLock::new();
    SHARED.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

#[derive(Debug)]
struct Headless;

impl Platform for Headless {
    fn pick_folder(&self, _: &str, _: Option<&Path>, reply: PickedFolder) {
        reply.send(None).ok();
    }
    fn pick_file(&self, _: &str, reply: PickedFolder) {
        reply.send(None).ok();
    }
    fn pick_files(&self, _: &str, _: Option<FileFilter>, reply: PickedFiles) {
        reply.send(Vec::new()).ok();
    }
    fn open_path(&self, _: &Path) -> Result<(), String> {
        Ok(())
    }
    fn reveal_path(&self, _: &Path) -> Result<(), String> {
        Ok(())
    }
    fn notify(&self, _: &str, _: &str) {}
    fn editor_focused(&self) -> bool {
        false
    }
    fn raise_editor(&self) {}
    fn exit(&self, _: i32) {}
    fn versions(&self) -> Versions {
        Versions::default()
    }
    fn open_view_window(&self, _: &str, _: &str, _: Closed) -> Result<(), String> {
        Ok(())
    }
    fn close_view_window(&self, _: &str) {}
    fn open_terminal_window(&self, _: &str, _: &str, _: Closed) -> Result<(), String> {
        Ok(())
    }
    fn close_terminal_window(&self, _: &str) {}
    fn headless(&self) -> bool {
        true
    }
}

struct Opened {
    app: Session,
    dir: PathBuf,
    events: Receiver<opsis_session::events::Addressed>,
    _temp: tempfile::TempDir,
}

async fn opened() -> Opened {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("project");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::copy(
        Path::new(EXAMPLE).join(PROJECT_FILE),
        dir.join(PROJECT_FILE),
    )
    .unwrap();
    let paths = AppPaths::new(&HostFolders {
        config: temp.path().join("config"),
        resources: temp.path().join("resources"),
        home: temp.path().to_path_buf(),
    })
    .unwrap();
    let app = Session::start(paths, Err(KeyError::NoKeyring), Arc::new(Headless))
        .await
        .unwrap();
    opsis_session::document::open(&app, &dir).await.unwrap();
    let events = app.subscribe();
    Opened {
        app,
        dir,
        events,
        _temp: temp,
    }
}

async fn edit_name(app: &Session, name: &str) {
    let source = ScriptSource::Text {
        name: "scripts/edit.lua".into(),
        source: format!("opsis.project.name = '{name}'"),
    };
    let outcome = run_script(app, source).unwrap().await.unwrap();
    assert!(outcome.ok, "{outcome:?}");
}

fn name(app: &Session) -> String {
    opsis_session::document::document_state(app)
        .unwrap()
        .workflow
        .name
}

fn write_name_on_disk(dir: &Path, name: &str) {
    let file = dir.join(PROJECT_FILE);
    let mut project: Value =
        serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    project["name"] = json!(name);
    let written = dir.join("project.opsis.json.new");
    std::fs::write(&written, serde_json::to_string_pretty(&project).unwrap()).unwrap();
    std::fs::rename(written, file).unwrap();
}

fn pending_question(app: &Session) -> bool {
    app.state::<AppState>().editor().disk_change.is_some()
}

async fn until(what: &str, mut done: impl FnMut() -> bool) {
    let started = Instant::now();
    while !done() {
        assert!(started.elapsed() < WITHIN, "{what} didn't happen");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn asked_anyone(events: &mut Receiver<opsis_session::events::Addressed>) -> bool {
    std::iter::from_fn(|| events.try_recv().ok()).any(|addressed| {
        matches!(
            addressed.event,
            SessionEvent::DiskChangeAsked(_) | SessionEvent::ConfirmClose(_)
        )
    })
}

#[test]
fn loads_a_changed_project_file_over_unsaved_edits() {
    runtime().block_on(async {
        let mut o = opened().await;
        edit_name(&o.app, "Mine").await;
        write_name_on_disk(&o.dir, "From disk");
        opsis_session::document::save(&o.app).await.unwrap();
        until("loading the file", || name(&o.app) == "From disk").await;
        until("answering the question", || !pending_question(&o.app)).await;
        assert!(!asked_anyone(&mut o.events));
        opsis_session::document::undo(&o.app).unwrap();
        assert_eq!(name(&o.app), "Mine");
    });
}

#[test]
fn waits_while_the_changed_file_isnt_a_project() {
    runtime().block_on(async {
        let mut o = opened().await;
        edit_name(&o.app, "Mine").await;
        std::fs::write(o.dir.join(PROJECT_FILE), "not a project").unwrap();
        assert!(!opsis_session::document::save(&o.app).await.unwrap());
        until("answering the question", || !pending_question(&o.app)).await;
        assert_eq!(name(&o.app), "Mine");
        assert_eq!(
            std::fs::read_to_string(o.dir.join(PROJECT_FILE)).unwrap(),
            "not a project"
        );
        assert!(!asked_anyone(&mut o.events));
    });
}

#[test]
fn exits_without_asking_and_saves_first() {
    runtime().block_on(async {
        let mut o = opened().await;
        let patch = serde_json::from_value(json!({ "app": { "confirmExit": true } })).unwrap();
        o.app
            .state::<AppState>()
            .patch_settings(&patch)
            .await
            .unwrap();
        edit_name(&o.app, "Saved on exit").await;
        let closing = tokio::time::timeout(WITHIN, opsis_session::exit::confirm(&o.app))
            .await
            .unwrap();
        assert!(closing);
        assert!(!asked_anyone(&mut o.events));
        let saved = std::fs::read_to_string(o.dir.join(PROJECT_FILE)).unwrap();
        assert!(saved.contains("Saved on exit"));
    });
}

#[test]
fn serves_the_socket_with_the_assistant_switched_off() {
    runtime().block_on(async {
        let o = opened().await;
        assert!(!o.app.state::<AppState>().settings().assistant.enabled);
        opsis_session::api::follow_setting(&o.app);
        until("listening", || {
            opsis_session::api::api_status(&o.app).listening
        })
        .await;
        opsis_session::api::stop(&o.app);
    });
}
