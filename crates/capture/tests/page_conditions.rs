mod support;

use std::sync::Arc;

use opsis_capture::take::{SignInError, SignIns, play_take};
use opsis_types::actions::{ActionKind, ActionLog};
use opsis_types::values::{BrowserAction, SessionStep};
use serde_json::{Value, json};
use support::{NoSignIns, serve, session, step, take_env};

const THEME_PAGE: &str = r#"<!doctype html><body><script>
const dark = matchMedia("(prefers-color-scheme: dark)");
const show = () => { document.body.textContent = `theme ${dark.matches ? "dark" : "light"}`; };
dark.addEventListener("change", show);
show();
</script></body>"#;

const LANGUAGE_PAGE: &str = r"<!doctype html><body><script>
document.body.textContent = `lang ${navigator.language} ${new Intl.NumberFormat().format(1234.5)}`;
</script></body>";

const NETWORK_PAGE: &str = r#"<!doctype html><body><script>
const show = async () => {
  const reached = await fetch(`/ping?${Math.random()}`).then(() => "reached", () => "unreachable");
  document.body.textContent = `online ${navigator.onLine} ${reached}`;
};
addEventListener("online", show);
addEventListener("offline", show);
show();
</script></body>"#;

const ACCOUNT_PAGE: &str = r"<!doctype html><body><script>
document.body.textContent = `cookie[${document.cookie}] token[${localStorage.getItem('token')}] tab[${sessionStorage.getItem('tab')}]`;
</script></body>";

struct Saved(String);

impl SignIns for Saved {
    fn read(&self, _: &str) -> Result<Option<String>, SignInError> {
        Ok(Some(self.0.clone()))
    }
}

const COUNTER_PAGE: &str = r#"<!doctype html><body><button id="add">Add item</button><p id="out">items 0</p><script>
let items = 0;
document.getElementById("add").addEventListener("click", () => {
  document.getElementById("out").textContent = `items ${++items}`;
});
</script></body>"#;

fn shows(id: &str, text: &str) -> SessionStep {
    step(
        BrowserAction::Assert,
        id,
        json!({ "text": text, "exact": true, "timeoutMs": 3000 }),
    )
}

async fn play(page: &'static str, browser: Value, steps: Vec<SessionStep>) {
    let port = serve(page).await;
    play_on(port, Arc::new(NoSignIns), browser, steps).await;
}

fn replayed(step: SessionStep) -> SessionStep {
    SessionStep {
        replayed: true,
        ..step
    }
}

async fn play_on(
    port: u16,
    sign_ins: Arc<dyn SignIns>,
    browser: Value,
    steps: Vec<SessionStep>,
) -> ActionLog {
    let temp = tempfile::tempdir().unwrap();
    let env = take_env(temp.path(), sign_ins);
    let navigate = step(
        BrowserAction::Navigate,
        "go",
        json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
    );
    let all = std::iter::once(navigate).chain(steps).collect();
    let played = play_take(&env, &session(browser, all), None, |_| {
        Box::pin(async { Ok(()) })
    })
    .await;
    match played.outcome {
        Ok(took) => took.log,
        Err(error) => panic!("the take failed: {error}"),
    }
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn switches_the_theme_the_page_sees_without_reloading() {
    play(
        THEME_PAGE,
        json!({ "colorScheme": "light" }),
        vec![
            shows("before", "theme light"),
            step(
                BrowserAction::SwitchTheme,
                "dark",
                json!({ "colorScheme": "dark" }),
            ),
            shows("after", "theme dark"),
            step(
                BrowserAction::SwitchTheme,
                "light",
                json!({ "colorScheme": "light" }),
            ),
            shows("back", "theme light"),
        ],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn changes_the_language_the_page_reads_when_it_reloads() {
    play(
        LANGUAGE_PAGE,
        json!({ "locale": "en-US" }),
        vec![
            shows("before", "lang en-US 1,234.5"),
            step(
                BrowserAction::ChangeLanguage,
                "german",
                json!({ "locale": "de-DE", "reload": true }),
            ),
            shows("after", "lang de-DE 1.234,5"),
            step(
                BrowserAction::ChangeLanguage,
                "french",
                json!({ "locale": "fr-FR", "reload": true }),
            ),
            shows("again", "lang fr-FR 1\u{202f}234,5"),
        ],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn takes_the_page_offline_and_back_online() {
    play(
        NETWORK_PAGE,
        json!({}),
        vec![
            shows("before", "online true reached"),
            step(
                BrowserAction::GoOffline,
                "off",
                json!({ "connection": "offline" }),
            ),
            shows("offline", "online false unreachable"),
            step(
                BrowserAction::GoOffline,
                "on",
                json!({ "connection": "online" }),
            ),
            shows("online", "online true reached"),
        ],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn signs_out_of_a_saved_sign_in_and_stays_signed_out_on_reload() {
    let port = serve(ACCOUNT_PAGE).await;
    let saved = json!({
        "cookies": [{ "name": "sid", "value": "1", "domain": "127.0.0.1", "path": "/", "expires": -1, "httpOnly": false, "secure": false, "sameSite": "Lax" }],
        "origins": [{ "origin": format!("http://127.0.0.1:{port}"), "localStorage": [{ "name": "token", "value": "t" }] }],
    })
    .to_string();
    let steps = vec![
        step(
            BrowserAction::SignIn,
            "login",
            json!({ "url": "http://127.0.0.1:1/login", "secretName": "shop" }),
        ),
        step(
            BrowserAction::Navigate,
            "again",
            json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
        ),
        step(
            BrowserAction::SetState,
            "tab",
            json!({ "script": "sessionStorage.setItem('tab', 'x');", "reload": true }),
        ),
        shows("signed-in", "cookie[sid=1] token[t] tab[x]"),
        step(BrowserAction::SignOut, "logout", json!({ "reload": true })),
        shows("signed-out", "cookie[] token[null] tab[null]"),
        step(
            BrowserAction::Navigate,
            "later",
            json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
        ),
        shows("still-out", "cookie[] token[null] tab[null]"),
    ];
    play_on(port, Arc::new(Saved(saved)), json!({}), steps).await;
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn replayed_steps_change_the_page_but_leave_nothing_in_the_take() {
    let port = serve(COUNTER_PAGE).await;
    let steps = vec![
        replayed(step(
            BrowserAction::Click,
            "earlier-click",
            json!({ "target": "Add item", "after": 0, "hoverMs": 0 }),
        )),
        replayed(step(
            BrowserAction::Chapter,
            "earlier-chapter",
            json!({ "title": "Earlier" }),
        )),
        step(BrowserAction::Chapter, "chapter", json!({ "title": "Now" })),
        shows("count", "items 1"),
    ];
    let log = play_on(port, Arc::new(NoSignIns), json!({}), steps).await;
    let logged: Vec<&str> = log
        .actions
        .iter()
        .filter_map(|action| action.node_id.as_deref())
        .collect();
    assert!(
        !logged.contains(&"earlier-click") && !logged.contains(&"earlier-chapter"),
        "{logged:?}"
    );
    let chapters: Vec<Option<&str>> = log
        .actions
        .iter()
        .filter(|action| action.kind == ActionKind::Chapter)
        .map(|action| action.label.as_deref())
        .collect();
    assert_eq!(chapters, [Some("Now")]);
}
