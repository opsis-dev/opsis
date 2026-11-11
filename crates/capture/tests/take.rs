mod support;

use opsis_capture::page_scripts::hiding_css;
use opsis_capture::take::{
    at_take_start, click_kind_of, landed_on_sign_in, trace_group_title, video_start,
};
use opsis_types::actions::{Action, ActionKind};
use opsis_types::values::{BrowserAction, SessionStep};
use proptest::prelude::*;
use serde_json::json;
use support::step;

#[test]
fn names_a_trace_group_after_what_the_action_acts_on() {
    assert_eq!(
        trace_group_title(
            "Navigate",
            &step(
                BrowserAction::Navigate,
                "n",
                json!({ "url": "https://x.test/login" })
            )
        ),
        "Navigate · https://x.test/login"
    );
    assert_eq!(
        trace_group_title(
            "Type",
            &step(
                BrowserAction::Type,
                "n",
                json!({ "target": "Username", "text": "tom" })
            )
        ),
        "Type · Username"
    );
    assert_eq!(
        trace_group_title(
            "Assert",
            &step(
                BrowserAction::Assert,
                "n",
                json!({ "text": "Welcome", "target": "" })
            )
        ),
        "Assert · Welcome"
    );
    assert_eq!(
        trace_group_title(
            "Wait",
            &step(BrowserAction::Wait, "n", json!({ "ms": 500 }))
        ),
        "Wait"
    );
}

#[test]
fn notices_a_take_that_lands_on_the_sign_in_page() {
    let sign_in = "https://x.test/login";
    let away = "https://x.test/app";
    let root = "https://www.saucedemo.com/";
    let inventory = "https://www.saucedemo.com/inventory.html";
    for (requested, current, page, expired) in [
        (away, "https://x.test/login/", sign_in, true),
        (away, "https://x.test/login/index.html", sign_in, true),
        (away, "https://x.test/login?next=app", sign_in, true),
        (away, "https://x.test/app", sign_in, false),
        (away, "not a url", sign_in, false),
        (
            "https://x.test/login?next=app",
            "https://x.test/login",
            sign_in,
            false,
        ),
        (inventory, root, root, true),
        (inventory, inventory, root, false),
        (root, root, root, false),
    ] {
        assert_eq!(
            landed_on_sign_in(requested, current, page),
            expired,
            "asked for {requested}, landed on {current}"
        );
    }
}

#[test]
fn leaves_the_actions_before_start_recording_out_of_the_video() {
    let steps = [
        step(BrowserAction::SignIn, "login", json!({})),
        step(
            BrowserAction::Navigate,
            "go",
            json!({ "url": "https://example.com" }),
        ),
        step(
            BrowserAction::Wait,
            "wait",
            json!({ "mode": "element", "target": "Board" }),
        ),
        step(BrowserAction::StartRecording, "start", json!({})),
        step(BrowserAction::Click, "open", json!({ "target": "Open" })),
        step(BrowserAction::StartRecording, "again", json!({})),
    ];
    let before: Vec<&str> = steps[..video_start(&steps)]
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(before, ["login", "go", "wait"]);
}

#[test]
fn films_from_the_first_action_without_start_recording() {
    let steps = [
        step(
            BrowserAction::Navigate,
            "go",
            json!({ "url": "https://example.com" }),
        ),
        step(BrowserAction::Click, "open", json!({ "target": "Open" })),
    ];
    assert_eq!(video_start(&steps), 0);
}

proptest! {
    #[test]
    fn films_a_passed_on_session_from_its_first_new_step(earlier in 0usize..6, new in 1usize..6) {
        let replayed = (0..earlier).map(|i| SessionStep {
            replayed: true,
            ..step(BrowserAction::Click, &format!("earlier{i}"), json!({ "target": "Open" }))
        });
        let fresh = (0..new).map(|i| step(BrowserAction::Click, &format!("new{i}"), json!({ "target": "Open" })));
        let steps: Vec<SessionStep> = replayed.chain(fresh).collect();
        prop_assert_eq!(video_start(&steps), earlier);
    }
}

#[test]
fn logs_each_pointer_action_as_its_kind() {
    for (kind, logged) in [
        (BrowserAction::Click, ActionKind::Click),
        (BrowserAction::Type, ActionKind::Type),
        (BrowserAction::Hover, ActionKind::Hover),
        (BrowserAction::Drag, ActionKind::Drag),
        (BrowserAction::Select, ActionKind::Focus),
    ] {
        assert_eq!(click_kind_of(kind), Some(logged), "{kind:?}");
    }
    assert_eq!(click_kind_of(BrowserAction::Navigate), None);
}

#[test]
fn puts_actions_before_the_camera_rolls_at_the_start_of_the_take() {
    let mut chapter = Action::new("c:chapter", ActionKind::Chapter, 3008.0);
    chapter.label = Some("Intro".into());
    let moved = at_take_start(chapter.clone());
    assert_eq!(moved, Action { t: 0.0, ..chapter });
    let mut nav = Action::new("n:nav", ActionKind::Navigate, 3009.0);
    nav.t_end = Some(3037.0);
    let moved = at_take_start(nav.clone());
    assert_eq!(
        moved,
        Action {
            t: 0.0,
            t_end: Some(0.0),
            ..nav
        }
    );
}

#[test]
fn hides_each_selector_with_its_own_rule_so_one_bad_selector_hides_nothing_less() {
    let css = hiding_css(&[".cookie-banner".to_owned(), "div[".to_owned()]);
    assert_eq!(
        css.split('\n').collect::<Vec<_>>(),
        [
            ".cookie-banner { display: none !important; visibility: hidden !important; }",
            "div[ { display: none !important; visibility: hidden !important; }"
        ]
    );
}
