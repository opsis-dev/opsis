use std::path::PathBuf;

use opsis_capture::browser::{Session, shell_in};
use opsis_capture::cdp::Arg;
use opsis_capture::locators::{TextPattern, first, role_selector};
use opsis_compile::nodes::browser::Open;
use serde_json::json;

fn engine_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME")
        .join(".config/Opsis/engine")
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn finds_a_role_through_the_page_script() {
    let temp = tempfile::tempdir().unwrap();
    let params: Open = serde_json::from_value(json!({})).unwrap();
    let session = Session::open(&params, &shell_in(&engine_dir()).unwrap(), temp.path())
        .await
        .unwrap();
    let html = "<button style='margin:40px'>Save draft</button><input placeholder='Name'>";
    session
        .tab
        .send(
            "Page.navigate",
            json!({ "url": format!("data:text/html,{html}") }),
        )
        .await
        .unwrap();
    for _ in 0..50 {
        let ready: String = session
            .tab
            .evaluate_main("document.readyState")
            .await
            .unwrap();
        if ready == "complete" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let selector = first(&role_selector("button", &TextPattern::exact("save draft")));
    let visible: bool = session
        .tab
        .injected_value("(injected, selector) => { const el = injected.querySelector(injected.parseSelector(selector), document, false); return !!el && injected.elementState(el, 'visible', true).matches; }", &[Arg::Value(json!(selector))])
        .await
        .unwrap();
    assert!(visible);
    let width: f64 = session.tab.evaluate_main("innerWidth").await.unwrap();
    assert_eq!(width, params.viewport_width as f64);
    session.close().await;
}
