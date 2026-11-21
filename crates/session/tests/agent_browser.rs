use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_folders::HostFolders;
use opsis_persistence::keys::KeyError;
use opsis_session::Session;
use opsis_session::paths::AppPaths;
use opsis_session::platform::{Closed, FileFilter, PickedFiles, PickedFolder, Platform, Versions};
use opsis_session::script::{ScriptSource, run_script};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const PAGE: &str = r#"<!doctype html><body><button onclick="document.getElementById('out').textContent='added'">Add item</button><p id="out">empty</p></body>"#;

#[derive(Debug)]
struct PicksBrowser(PathBuf);

impl Platform for PicksBrowser {
    fn pick_folder(&self, _: &str, _: Option<&Path>, _: PickedFolder) {}
    fn pick_file(&self, _: &str, reply: PickedFolder) {
        if reply.send(Some(self.0.clone())).is_err() {
            log::debug!("no one waits for the browser pick");
        }
    }
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

fn engine_browser() -> PathBuf {
    let engine = std::env::var_os("OPSIS_ENGINE_DIR").map_or_else(
        || opsis_golden::home().join(".config/Opsis/engine"),
        PathBuf::from,
    );
    opsis_engine::paths::downloaded_browser(&engine).unwrap()
}

async fn serve(page: &'static str) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut request = [0; 4096];
            if socket.read(&mut request).await.is_ok() {
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{page}",
                    page.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        }
    });
    port
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn a_script_opens_the_project_browser_on_its_flow_checks_on_and_closes_it() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(&HostFolders {
        config: temp.path().join("config"),
        resources: temp.path().join("resources"),
        home: temp.path().to_path_buf(),
    })
    .unwrap();
    let app = Session::start(
        paths,
        Err(KeyError::NoKeyring),
        Arc::new(PicksBrowser(engine_browser())),
    )
    .await
    .unwrap();
    assert!(
        opsis_session::engine::choose_browser(app.clone())
            .await
            .unwrap()
    );
    opsis_session::projects::create_project(
        app.clone(),
        "Shop".into(),
        "blank".into(),
        Some(temp.path().to_string_lossy().into_owned()),
    )
    .await
    .unwrap()
    .unwrap();
    let port = serve(PAGE).await;
    let source = format!(
        r#"
local open = opsis.nodes.add {{ type = "browser.open" }}
local go = opsis.nodes.add {{ type = "browser.navigate", params = {{ url = "http://127.0.0.1:{port}/", after = 0 }} }}
local add = opsis.nodes.add {{ type = "browser.click", params = {{ target = "Add item", after = 0, hoverMs = 0 }} }}
local added = opsis.nodes.add {{ type = "browser.assert", params = {{ text = "added", timeoutMs = 2000 }} }}
local missing = opsis.nodes.add {{ type = "browser.assert", params = {{ text = "nowhere on the page", timeoutMs = 1000 }} }}
opsis.edges.connect(open, "session", go, "session")
opsis.edges.connect(go, "session", add, "session")
opsis.edges.connect(add, "session", added, "session")
opsis.edges.connect(added, "session", missing, "session")
local opened = opsis.browser.open(go)
local checked = opsis.browser.check(missing)
local open_state = opsis.browser.state()
opsis.browser.close()
return {{
  opened = opened,
  states = {{ checked.actions[1].state, checked.actions[2].state, checked.actions[3].state }},
  at = checked.at,
  still = open_state.at,
  added = added.id,
  go = go.id,
  closed = opsis.browser.state() == nil,
}}
"#
    );
    let outcome = run_script(
        &app,
        ScriptSource::Text {
            name: "scripts/browser.lua".into(),
            source,
        },
        None,
    )
    .unwrap()
    .await
    .unwrap();
    assert!(outcome.ok, "{outcome:?}");
    let value: Value = outcome.value.unwrap();
    assert!(
        value["opened"]["address"]
            .as_str()
            .unwrap()
            .starts_with("ws://127.0.0.1:")
    );
    assert_eq!(value["opened"]["at"], value["go"]);
    assert_eq!(
        value["opened"]["page"],
        json!(format!("http://127.0.0.1:{port}/"))
    );
    assert_eq!(value["states"], json!(["passed", "passed", "failed"]));
    assert_eq!(value["at"], value["added"]);
    assert_eq!(value["still"], value["added"]);
    assert_eq!(value["closed"], json!(true));
}
