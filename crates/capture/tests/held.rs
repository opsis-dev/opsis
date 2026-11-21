mod support;

use std::sync::Arc;

use opsis_capture::held::hold;
use opsis_types::values::{ActionState, BrowserAction};
use serde_json::json;
use support::{NoSignIns, serve, session, step, take_env};

const PAGE: &str = r#"<!doctype html><body><button id="add" onclick="document.getElementById('out').textContent='added'">Add item</button><p id="out">empty</p></body>"#;

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn holds_a_played_browser_and_checks_the_actions_after_it() {
    let port = serve(PAGE).await;
    let temp = tempfile::tempdir().unwrap();
    let go = step(
        BrowserAction::Navigate,
        "go",
        json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
    );
    let held = hold(
        take_env(temp.path(), Arc::new(NoSignIns)),
        session(json!({}), vec![go]),
    )
    .await
    .unwrap();
    assert!(
        held.address().starts_with("ws://127.0.0.1:"),
        "{}",
        held.address()
    );
    assert_eq!(
        held.page().await.unwrap(),
        format!("http://127.0.0.1:{port}/")
    );

    let click = step(
        BrowserAction::Click,
        "add",
        json!({ "target": "Add item", "after": 0, "hoverMs": 0 }),
    );
    let added = step(
        BrowserAction::Assert,
        "added",
        json!({ "text": "added", "timeoutMs": 2000 }),
    );
    let missing = step(
        BrowserAction::Assert,
        "missing",
        json!({ "text": "nowhere on the page", "timeoutMs": 1000 }),
    );
    let after = step(
        BrowserAction::Assert,
        "after",
        json!({ "text": "added", "timeoutMs": 1000 }),
    );
    let actions = held
        .check(vec![click, added, missing, after])
        .await
        .unwrap();
    let states: Vec<_> = actions
        .iter()
        .map(|action| (action.node_id.as_str(), action.state))
        .collect();
    assert_eq!(
        states,
        [
            ("add", ActionState::Passed),
            ("added", ActionState::Passed),
            ("missing", ActionState::Failed),
            ("after", ActionState::NotReached),
        ]
    );
    assert!(actions[2].message.is_some());

    let again = step(
        BrowserAction::Assert,
        "again",
        json!({ "text": "added", "timeoutMs": 1000 }),
    );
    let actions = held.check(vec![again]).await.unwrap();
    assert_eq!(actions[0].state, ActionState::Passed);
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn reports_the_action_that_failed_while_opening() {
    let port = serve(PAGE).await;
    let temp = tempfile::tempdir().unwrap();
    let failed = hold(
        take_env(temp.path(), Arc::new(NoSignIns)),
        session(
            json!({}),
            vec![
                step(
                    BrowserAction::Navigate,
                    "go",
                    json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
                ),
                step(
                    BrowserAction::Assert,
                    "missing",
                    json!({ "text": "nowhere on the page", "timeoutMs": 1000 }),
                ),
            ],
        ),
    )
    .await
    .unwrap_err();
    let states: Vec<_> = failed
        .actions
        .iter()
        .map(|action| (action.node_id.as_str(), action.state))
        .collect();
    assert_eq!(
        states,
        [
            ("go", ActionState::Passed),
            ("missing", ActionState::Failed)
        ]
    );
    let missing = &failed.actions[1];
    assert_eq!(
        failed.message,
        format!("{}: {}", missing.label, missing.message.as_deref().unwrap())
    );
}
