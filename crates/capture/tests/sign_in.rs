mod support;

use std::sync::Arc;

use opsis_capture::executors::sign_in_stamp;
use opsis_capture::take::{SignInError, SignIns, StepError, play_take};
use opsis_types::values::{BrowserAction, SessionStep};
use serde_json::json;
use support::{serve, session, step, take_env};

const PAGE: &str = "<!doctype html><body><script>const runs = localStorage.getItem('runs'); document.body.textContent = `${document.cookie} ${localStorage.getItem('token')} runs ${runs}`;</script>";

struct Saved(Option<String>);

impl SignIns for Saved {
    fn read(&self, _: &str) -> Result<Option<String>, SignInError> {
        Ok(self.0.clone())
    }
}

async fn play(saved: Option<String>, steps: Vec<SessionStep>) -> Result<(), StepError> {
    let temp = tempfile::tempdir().unwrap();
    let env = take_env(temp.path(), Arc::new(Saved(saved)));
    let session = session(json!({}), steps);
    play_take(&env, &session, None, |_| Box::pin(async { Ok(()) }))
        .await
        .outcome
        .map(drop)
}

fn sign_in() -> SessionStep {
    SessionStep {
        label: Some("Sign in to the shop".to_owned()),
        ..step(
            BrowserAction::SignIn,
            "login",
            json!({ "url": "http://127.0.0.1:1/login", "secretName": "shop" }),
        )
    }
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
    let port = serve(PAGE).await;
    let steps = vec![
        sign_in(),
        step(
            BrowserAction::Navigate,
            "go",
            json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
        ),
        step(
            BrowserAction::Assert,
            "check",
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
    let port = serve(PAGE).await;
    let url = format!("http://127.0.0.1:{port}/");
    let steps = vec![
        step(
            BrowserAction::SetState,
            "seed",
            json!({ "script": "localStorage.setItem('runs', String(Number(localStorage.getItem('runs') ?? 0) + 1));" }),
        ),
        step(
            BrowserAction::Navigate,
            "go",
            json!({ "url": url, "after": 0 }),
        ),
        step(
            BrowserAction::Navigate,
            "again",
            json!({ "url": url, "after": 0 }),
        ),
        step(
            BrowserAction::Assert,
            "check",
            json!({ "text": "runs 1", "timeoutMs": 2000 }),
        ),
    ];
    play(None, steps).await.unwrap();
}

#[test]
fn stamps_a_sign_in_by_its_saved_file_and_by_nothing_before_it_is_saved() {
    let dir = tempfile::tempdir().unwrap();
    let params = json!({ "secretName": "acct" }).as_object().unwrap().clone();
    assert!(sign_in_stamp(dir.path(), &params).is_empty());
    let file = opsis_types::secrets::secret_file(dir.path(), "acct");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "one").unwrap();
    let first = sign_in_stamp(dir.path(), &params);
    std::fs::write(&file, "a longer one").unwrap();
    assert_eq!(first.len(), 1);
    assert_ne!(sign_in_stamp(dir.path(), &params), first);
}
