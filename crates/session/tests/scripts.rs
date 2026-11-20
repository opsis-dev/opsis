use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use opsis_folders::HostFolders;
use opsis_session::Session;
use opsis_session::paths::AppPaths;
use opsis_session::platform::{Closed, FileFilter, PickedFiles, PickedFolder, Platform, Versions};
use opsis_session::script::{
    ScriptSource, ScriptStartError, run_script, script_status, stop_script,
};
use opsis_session::state::AppState;
use opsis_store::keys::KeyError;
use opsis_types::script::ScriptOutcome;
use opsis_types::workflow::Workflow;

#[derive(Debug)]
struct Quiet;

impl Platform for Quiet {
    fn pick_folder(&self, _: &str, _: Option<&Path>, _: PickedFolder) {}
    fn pick_file(&self, _: &str, _: PickedFolder) {}
    fn pick_files(&self, _: &str, _: Option<FileFilter>, _: PickedFiles) {}
    fn open_path(&self, _: &Path) -> Result<(), String> {
        Ok(())
    }
    fn reveal_path(&self, _: &Path) -> Result<(), String> {
        Ok(())
    }
    fn notify(&self, _: &str, _: &str) {}
    fn editor_focused(&self) -> bool {
        true
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
}

struct Opened {
    app: Session,
    dir: PathBuf,
    _temp: tempfile::TempDir,
}

const EXAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/todomvc");

async fn opened() -> Opened {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("project");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::copy(
        Path::new(EXAMPLE).join("project.opsis.json"),
        dir.join("project.opsis.json"),
    )
    .unwrap();
    let paths = AppPaths::new(&HostFolders {
        config: temp.path().join("config"),
        resources: temp.path().join("resources"),
        home: temp.path().to_path_buf(),
    })
    .unwrap();
    let app = Session::start(paths, Err(KeyError::NoKeyring), Arc::new(Quiet))
        .await
        .unwrap();
    opsis_session::document::open(&app, &dir).await.unwrap();
    Opened {
        app,
        dir,
        _temp: temp,
    }
}

fn workflow(app: &Session) -> Workflow {
    opsis_session::document::document_state(app)
        .unwrap()
        .workflow
}

fn text(source: &str) -> ScriptSource {
    ScriptSource::Text {
        name: "scripts/test.lua".into(),
        source: source.into(),
    }
}

async fn ended(app: &Session, source: &str) -> ScriptOutcome {
    run_script(app, text(source)).unwrap().await.unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn keeps_a_finished_scripts_edits_as_one_undo_step() {
    let o = opened().await;
    let outcome = ended(
        &o.app,
        "opsis.project.name = 'One'\nopsis.nodes.open.label = 'Two'\nreturn opsis.project.name",
    )
    .await;
    assert!(outcome.ok, "{outcome:?}");
    assert_eq!(outcome.value, Some(serde_json::json!("One")));
    assert!(!outcome.rolled_back);
    assert_eq!(workflow(&o.app).name, "One");
    opsis_session::document::undo(&o.app).unwrap();
    assert_eq!(workflow(&o.app).name, "Sample: to-do app");
    assert_eq!(workflow(&o.app).node("open").unwrap().label, None);
}

#[tokio::test(flavor = "multi_thread")]
async fn rolls_back_a_script_that_fails_and_names_its_line() {
    let o = opened().await;
    let before = workflow(&o.app);
    let outcome = ended(
        &o.app,
        "opsis.project.name = 'Broken'\n\nerror('stop here')",
    )
    .await;
    assert!(!outcome.ok);
    assert!(outcome.rolled_back);
    assert_eq!(outcome.failure.as_ref().unwrap().line, Some(3));
    assert_eq!(workflow(&o.app), before);
    let state = opsis_session::document::document_state(&o.app).unwrap();
    assert!(!state.can_undo && !state.can_redo);
}

#[tokio::test(flavor = "multi_thread")]
async fn rolls_back_a_stopped_script_refuses_a_second_and_holds_undo_meanwhile() {
    let o = opened().await;
    let before = workflow(&o.app);
    let outcome = run_script(
        &o.app,
        text("opsis.project.name = 'Half'\nwhile true do end"),
    )
    .unwrap();
    while workflow(&o.app).name != "Half" {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(
        run_script(&o.app, text("return 1")).unwrap_err(),
        ScriptStartError::Busy
    );
    assert!(opsis_session::document::undo(&o.app).is_err());
    assert!(script_status(&o.app).running);
    assert!(
        o.app
            .state::<AppState>()
            .tasks
            .labels()
            .contains(&"Running a script".to_owned())
    );
    stop_script(&o.app);
    let outcome = tokio::time::timeout(Duration::from_secs(2), outcome)
        .await
        .unwrap()
        .unwrap();
    assert!(outcome.stopped && outcome.rolled_back && !outcome.ok);
    assert_eq!(workflow(&o.app), before);
    assert_eq!(
        o.app.state::<AppState>().tasks.labels(),
        Vec::<String>::new()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn runs_a_file_from_the_scripts_folder_and_refuses_a_file_setting_outside() {
    let o = opened().await;
    std::fs::write(
        o.dir.join("scripts/add.lua"),
        "local n = opsis.nodes.add { type = 'media.import' }\nn.params.path = '/etc/hostname'\n",
    )
    .unwrap();
    let outcome = run_script(&o.app, ScriptSource::File("scripts/add.lua".into()))
        .unwrap()
        .await
        .unwrap();
    let failure = outcome.failure.unwrap();
    assert_eq!(failure.file, "scripts/add.lua");
    assert_eq!(failure.line, Some(2));
    assert!(failure.message.contains("path:"), "{}", failure.message);
    assert!(outcome.rolled_back);
    assert_eq!(
        run_script(&o.app, ScriptSource::File("../escape.lua".into())).unwrap_err(),
        ScriptStartError::NotAScript("../escape.lua".into())
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn stops_the_script_before_opening_another_project() {
    let o = opened().await;
    let outcome = run_script(&o.app, text("opsis.project.name = 'Mid'\nopsis.sleep(60)")).unwrap();
    while workflow(&o.app).name != "Mid" {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let other = o.dir.with_file_name("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::copy(
        o.dir.join("project.opsis.json"),
        other.join("project.opsis.json"),
    )
    .unwrap();
    opsis_session::document::open(&o.app, &other).await.unwrap();
    let outcome = outcome.await.unwrap();
    assert!(outcome.stopped && outcome.rolled_back);
    let saved: Workflow =
        serde_json::from_str(&std::fs::read_to_string(o.dir.join("project.opsis.json")).unwrap())
            .unwrap();
    assert_eq!(saved.name, "Sample: to-do app");
}
