use std::path::PathBuf;
use std::sync::Arc;

use opsis_capture::take::{SignInError, SignIns, TakeEnv, play_take};
use opsis_compile::nodes::browser::OPEN_BROWSER_TYPE;
use opsis_compile::registry::default_params;
use opsis_engine::paths::downloaded_browser;
use opsis_types::actions::{ActionKind, ActionLog, ClickButton};
use opsis_types::values::{BrowserAction, BrowserSessionValue, SessionBrowser, SessionStep};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

const PAGE: &str = r#"<!doctype html><body style="margin:0"><button id="folder" style="margin:200px;width:240px;height:80px">Reports folder</button><button id="out">click 0 · double 0 · menu 0</button><a href="/settings" id="settings">Settings</a> <a href="/reports">Reports</a><script>
document.getElementById("settings").addEventListener("click", (e) => { e.preventDefault(); history.pushState({}, "", "/settings"); });
const n = { click: 0, double: 0, menu: 0 };
const show = () => { document.getElementById("out").textContent = `click ${n.click} · double ${n.double} · menu ${n.menu}`; };
const folder = document.getElementById("folder");
folder.addEventListener("click", () => { n.click++; show(); });
folder.addEventListener("dblclick", () => { n.double++; show(); });
folder.addEventListener("contextmenu", (e) => { e.preventDefault(); n.menu++; show(); });
</script>"#;

struct NoSignIns;

impl SignIns for NoSignIns {
    fn read(&self, _: &str) -> Result<Option<String>, SignInError> {
        Ok(None)
    }
}

async fn serve() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut request = [0; 4096];
            if socket.read(&mut request).await.is_ok() {
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{PAGE}",
                    PAGE.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        }
    });
    port
}

fn step(kind: BrowserAction, id: &str, params: Value) -> SessionStep {
    let mut all = default_params(kind.kind());
    all.extend(params.as_object().cloned().unwrap_or_default());
    SessionStep {
        id: id.to_owned(),
        kind,
        label: None,
        params: all,
    }
}

async fn play(steps: Vec<SessionStep>) -> ActionLog {
    let temp = tempfile::tempdir().unwrap();
    let engine = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine");
    let env = TakeEnv {
        project_dir: temp.path().to_path_buf(),
        executable: downloaded_browser(&engine).unwrap(),
        temp_dir: temp.path().to_path_buf(),
        live: None,
        trace: false,
        sign_ins: Arc::new(NoSignIns),
        cancel: CancellationToken::new(),
        log: Arc::new(|_, _| {}),
        step: Arc::new(|_, _, _| {}),
    };
    let session = BrowserSessionValue {
        browser: SessionBrowser {
            node_id: "browser".into(),
            params: default_params(OPEN_BROWSER_TYPE),
        },
        steps,
        address: None,
    };
    play_take(&env, &session, None, |_| Box::pin(async { Ok(()) }))
        .await
        .outcome
        .unwrap()
        .log
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn clicks_double_clicks_and_right_clicks_as_the_page_counts_them() {
    let port = serve().await;
    let click = |id: &str, extra: Value| {
        let mut params = json!({ "target": "Reports folder", "after": 0, "hoverMs": 0 });
        params
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        step(BrowserAction::Click, id, params)
    };
    let log = play(vec![
        step(
            BrowserAction::Navigate,
            "go",
            json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
        ),
        click("once", json!({})),
        click("twice", json!({ "double": true })),
        click("menu", json!({ "button": "right" })),
        step(
            BrowserAction::Assert,
            "check",
            json!({ "text": "click 3 · double 1 · menu 1", "timeoutMs": 2000 }),
        ),
    ])
    .await;
    let clicks: Vec<_> = log
        .actions
        .iter()
        .filter(|a| a.kind == ActionKind::Click)
        .collect();
    assert_eq!(clicks.len(), 3);
    assert_eq!(
        clicks
            .iter()
            .map(|a| (a.t_second_down.is_some(), a.button))
            .collect::<Vec<_>>(),
        [
            (false, None),
            (true, None),
            (false, Some(ClickButton::Right))
        ]
    );
    let double = clicks[1];
    assert!(double.t_down.unwrap() < double.t_second_down.unwrap());
    assert!(double.t_second_down.unwrap() <= double.t);
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn logs_every_address_the_page_shows_with_route_changes_in_place() {
    let port = serve().await;
    let site = format!("http://127.0.0.1:{port}");
    let click = |id: &str, target: &str| {
        step(
            BrowserAction::Click,
            id,
            json!({ "target": target, "after": 300, "hoverMs": 0 }),
        )
    };
    let log = play(vec![
        step(
            BrowserAction::Navigate,
            "go",
            json!({ "url": format!("{site}/inbox"), "after": 0 }),
        ),
        click("settings", "Settings"),
        click("reports", "Reports"),
        step(
            BrowserAction::Assert,
            "check",
            json!({ "text": "Reports folder", "timeoutMs": 2000 }),
        ),
    ])
    .await;
    let urls: Vec<&str> = log.addresses.iter().map(|a| a.url.as_str()).collect();
    assert_eq!(
        urls,
        [
            format!("{site}/inbox"),
            format!("{site}/settings"),
            format!("{site}/reports")
        ]
    );
    assert!(log.addresses.windows(2).all(|w| w[0].t <= w[1].t));
    let clicked = |id: &str| {
        log.actions
            .iter()
            .find(|a| a.id.starts_with(id) && a.kind == ActionKind::Click)
            .unwrap()
            .t
    };
    assert!(log.addresses[1].t >= clicked("settings") - 100.0);
    assert!(log.addresses[2].t >= clicked("reports") - 100.0);
}
