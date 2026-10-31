mod support;

use opsis_capture::browser::{Session, check_browser};
use opsis_capture::cdp::Arg;
use opsis_capture::locators::{TextPattern, first, role_selector};
use opsis_compile::nodes::browser::Open;
use opsis_engine::paths::downloaded_browser;
use serde_json::json;
use support::engine_dir;

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn accepts_the_downloaded_browser_and_refuses_another_program() {
    let temp = tempfile::tempdir().unwrap();
    let browser = downloaded_browser(&engine_dir()).unwrap();
    let version = check_browser(&browser, temp.path()).await.unwrap();
    assert!(version.split('.').count() == 4, "{version}");
    assert!(
        check_browser(&opsis_golden::ffmpeg(), temp.path())
            .await
            .is_err()
    );
}

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn finds_a_role_through_the_page_script() {
    let temp = tempfile::tempdir().unwrap();
    let params: Open = serde_json::from_value(json!({})).unwrap();
    let session = Session::open(
        &params,
        &downloaded_browser(&engine_dir()).unwrap(),
        temp.path(),
    )
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
    assert_eq!(width, f64::from(params.viewport_width));
    session.close().await;
}
