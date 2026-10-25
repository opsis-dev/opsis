use std::path::PathBuf;
use std::sync::Arc;

use opsis_capture::browser::shell_in;
use opsis_capture::take::{SignIns, StepError, TakeEnv, play_take};
use opsis_compile::registry::default_params;
use opsis_types::values::{BrowserSessionValue, SessionBrowser, SessionStep};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

const PAGE: &str = "<!doctype html><body><script>const runs = localStorage.getItem('runs'); document.body.textContent = `${document.cookie} ${localStorage.getItem('token')} runs ${runs}`;</script>";

struct Saved(Option<String>);

impl SignIns for Saved {
    fn read(&self, _: &str) -> Result<Option<String>, String> {
        Ok(self.0.clone())
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
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{PAGE}",
                    PAGE.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        }
    });
    port
}

fn step(kind: &str, id: &str, label: Option<&str>, params: Value) -> SessionStep {
    let mut all = default_params(kind);
    all.extend(params.as_object().cloned().unwrap_or_default());
    SessionStep {
        id: id.to_owned(),
        kind: kind.to_owned(),
        label: label.map(str::to_owned),
        params: all,
    }
}

async fn play(saved: Option<String>, steps: Vec<SessionStep>) -> Result<(), StepError> {
    let temp = tempfile::tempdir().unwrap();
    let engine = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine");
    let env = TakeEnv {
        project_dir: temp.path().to_path_buf(),
        executable: shell_in(&engine).unwrap(),
        temp_dir: temp.path().to_path_buf(),
        live: None,
        trace: false,
        sign_ins: Arc::new(Saved(saved)),
        cancel: CancellationToken::new(),
        log: Arc::new(|_, _| {}),
        step: Arc::new(|_, _, _| {}),
    };
    let session = BrowserSessionValue {
        browser: SessionBrowser {
            node_id: "browser".into(),
            params: default_params("browser.open"),
        },
        steps,
    };
    play_take(&env, &session, None, |_| Box::pin(async { Ok(()) }))
        .await
        .outcome
        .map(drop)
}

fn sign_in() -> SessionStep {
    step(
        "browser.signIn",
        "login",
        Some("Sign in to the shop"),
        json!({ "url": "http://127.0.0.1:1/login", "secretName": "shop" }),
    )
}

fn saved(port: u16) -> String {
    json!({
        "cookies": [{ "name": "sid", "value": "1", "domain": "127.0.0.1", "path": "/", "expires": -1, "httpOnly": false, "secure": false, "sameSite": "Lax" }],
        "origins": [{ "origin": format!("http://127.0.0.1:{port}"), "localStorage": [{ "name": "token", "value": "t" }] }],
    })
    .to_string()
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn signs_in_with_the_saved_cookies_and_page_storage() {
    let port = serve().await;
    let steps = vec![
        sign_in(),
        step(
            "browser.navigate",
            "go",
            None,
            json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
        ),
        step(
            "browser.assert",
            "check",
            None,
            json!({ "text": "sid=1 t", "timeoutMs": 2000 }),
        ),
    ];
    play(Some(saved(port)), steps).await.unwrap();
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn asks_for_the_sign_in_when_it_isnt_saved() {
    let error = play(None, vec![sign_in()]).await.unwrap_err();
    assert_eq!(
        error.to_string(),
        "The sign-in “shop” isn't saved yet. Run “Sign in to the shop” first."
    );
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn runs_a_script_set_before_the_first_page_once_per_site() {
    let port = serve().await;
    let url = format!("http://127.0.0.1:{port}/");
    let steps = vec![
        step(
            "browser.setState",
            "seed",
            None,
            json!({ "script": "localStorage.setItem('runs', String(Number(localStorage.getItem('runs') ?? 0) + 1));" }),
        ),
        step(
            "browser.navigate",
            "go",
            None,
            json!({ "url": url, "after": 0 }),
        ),
        step(
            "browser.navigate",
            "again",
            None,
            json!({ "url": url, "after": 0 }),
        ),
        step(
            "browser.assert",
            "check",
            None,
            json!({ "text": "runs 1", "timeoutMs": 2000 }),
        ),
    ];
    play(None, steps).await.unwrap();
}
