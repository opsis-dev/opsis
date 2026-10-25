use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use opsis_capture::browser::{Session, shell_in};
use opsis_capture::cdp::CaptureError;
use opsis_capture::page::{Lifecycle, NAVIGATION_TIMEOUT};
use opsis_capture::selectors::{ResolveError, ResolveOptions, Resolved, Target, resolve_target};
use opsis_compile::nodes::browser::{Open, WaitUntil};
use opsis_types::actions::Point;
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn engine_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME")
        .join(".config/Opsis/engine")
}

struct Page {
    session: Session,
    _temp: tempfile::TempDir,
}

async fn page(html: &str) -> Page {
    let temp = tempfile::tempdir().unwrap();
    let params: Open = serde_json::from_value(json!({})).unwrap();
    let session = Session::open(&params, &shell_in(&engine_dir()).unwrap(), temp.path())
        .await
        .unwrap();
    let lifecycle = Lifecycle::start(&session.tab).await.unwrap();
    let url = format!("data:text/html;base64,{}", STANDARD.encode(html));
    lifecycle
        .navigate(&session.tab, &url, WaitUntil::Load, NAVIGATION_TIMEOUT)
        .await
        .unwrap();
    Page {
        session,
        _temp: temp,
    }
}

async fn resolve(
    page: &Page,
    raw: &str,
    hoisted: &HashMap<String, String>,
    timeout: Option<Duration>,
    cancel: &CancellationToken,
) -> Result<Resolved, ResolveError> {
    resolve_target(
        &page.session.tab,
        raw,
        &ResolveOptions {
            log: &|_| {},
            cancel,
            hoisted,
            timeout,
        },
    )
    .await
}

async fn found_by(html: &str, raw: &str) -> String {
    let page = page(html).await;
    let found = resolve(&page, raw, &HashMap::new(), None, &CancellationToken::new()).await;
    page.session.close().await;
    match found {
        Ok(found) => found.by,
        Err(ResolveError::NotFound(error)) => panic!("{error}"),
        Err(ResolveError::Other(error)) => panic!("{error}"),
    }
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn prefers_an_exact_button_over_text_that_merely_contains_the_name() {
    assert_eq!(
        found_by("<p>Save the draft first</p><button>Save</button>", "Save").await,
        r#"role=button name="Save""#
    );
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn falls_through_the_roles_to_the_first_rung_that_shows_something() {
    assert_eq!(
        found_by(
            "<label for=e>Email</label><input id=e type=checkbox>",
            "Email"
        )
        .await,
        r#"label="Email""#
    );
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn keeps_looking_until_the_element_appears() {
    let html = "<script>setTimeout(() => { const d = document.createElement('div'); d.dataset.testid = 'cart'; d.textContent = 'basket'; document.body.append(d); }, 400)</script>";
    assert_eq!(found_by(html, "cart").await, r#"testid="cart""#);
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn uses_a_held_element_a_point_or_a_selector_as_given() {
    let page = page("<button id=add>Add</button>").await;
    let cancel = CancellationToken::new();
    let hoisted = HashMap::from([("Row".to_owned(), "#row >> nth=0".to_owned())]);
    let held = resolve(&page, "Row", &hoisted, None, &cancel)
        .await
        .ok()
        .unwrap();
    assert_eq!(
        (held.target, held.by.as_str()),
        (Target::Element("#row >> nth=0".into()), "hoisted")
    );
    let point = resolve(&page, "120, 48", &hoisted, None, &cancel)
        .await
        .ok()
        .unwrap();
    assert_eq!(
        (point.target, point.by.as_str()),
        (Target::Point(Point { x: 120.0, y: 48.0 }), "point")
    );
    let css = resolve(&page, "css=#add", &hoisted, None, &cancel)
        .await
        .ok()
        .unwrap();
    assert_eq!(
        (css.target, css.by.as_str()),
        (Target::Element("#add >> nth=0".into()), "css=#add")
    );
    page.session.close().await;
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn lists_what_it_tried_when_nothing_turns_up_in_time() {
    let page = page("<button>Save</button>").await;
    let missed = resolve(
        &page,
        "Missing",
        &HashMap::new(),
        Some(Duration::ZERO),
        &CancellationToken::new(),
    )
    .await;
    page.session.close().await;
    let Err(ResolveError::NotFound(error)) = missed else {
        panic!("expected the element to be missing")
    };
    assert_eq!(
        error.tried.first().map(String::as_str),
        Some(r#"role=button name="Missing""#)
    );
    assert_eq!(
        error.tried.last().map(String::as_str),
        Some(r#"text "Missing""#)
    );
    assert!(error.to_string().contains(r#"<button> "Save""#), "{error}");
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn stops_when_the_take_is_cancelled() {
    let page = page("<button>Save</button>").await;
    let cancel = CancellationToken::new();
    cancel.cancel();
    let stopped = resolve(&page, "Missing", &HashMap::new(), None, &cancel).await;
    page.session.close().await;
    assert!(matches!(
        stopped,
        Err(ResolveError::Other(CaptureError::Cancelled))
    ));
}
