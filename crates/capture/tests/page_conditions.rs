mod support;

use std::sync::Arc;

use opsis_capture::take::play_take;
use opsis_types::values::{BrowserAction, SessionStep};
use serde_json::{Value, json};
use support::{NoSignIns, serve, session, step, take_env};

const THEME_PAGE: &str = r#"<!doctype html><body><script>
const dark = matchMedia("(prefers-color-scheme: dark)");
const show = () => { document.body.textContent = `theme ${dark.matches ? "dark" : "light"}`; };
dark.addEventListener("change", show);
show();
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
    let temp = tempfile::tempdir().unwrap();
    let env = take_env(temp.path(), Arc::new(NoSignIns));
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
    if let Err(error) = played.outcome {
        panic!("the take failed: {error}");
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
