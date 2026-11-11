use indexmap::IndexMap;
use opsis_compile::compile::{compile_workflow, interpolated_params};
use opsis_compile::node::CardInputs;
use opsis_compile::nodes::browser::nodes;
use opsis_compile::registry::node_def;
use opsis_compile::vars::{PAUSE_MARK, Vars};
use opsis_types::cards::{Icon, Tone};
use opsis_types::values::{BrowserSessionValue, CompileValue};
use opsis_types::workflow::Workflow;
use serde_json::{Map, Value, json};

use crate::helpers::{Graph, card_texts, node_card, parse_params, problem};

fn icon_text(kind: &str, params: Value) -> Vec<(Icon, String)> {
    node_card(kind, params, CardInputs::default())
        .into_iter()
        .map(|r| (r.icon, r.text))
        .collect()
}

#[test]
fn shows_what_browser_actions_do_on_their_cards() {
    let typed = icon_text(
        "browser.type",
        json!({ "target": "Username", "targetRole": "textbox", "text": "tom¦smith", "pressEnter": true, "after": 500, "cluster": "login" }),
    );
    let expected = [
        (Icon::Crosshair, "“Username” field"),
        (Icon::Keyboard, "“tom … smith”, then Enter"),
        (Icon::Timer, "Pause 0.5 s"),
        (Icon::ZoomIn, "Zoom group “login”"),
    ];
    assert_eq!(typed, expected.map(|(i, t)| (i, t.to_owned())));
    let click = node_card(
        "browser.click",
        json!({ "target": "" }),
        CardInputs::default(),
    )[0]
    .clone();
    assert_eq!(
        (click.text.as_str(), click.tone),
        ("Pick an element", Some(Tone::Warn))
    );
    let drag = |params: Value| {
        let mut all = json!({ "target": "Deal card", "after": 0 });
        all.as_object_mut()
            .unwrap()
            .extend(params.as_object().unwrap().clone());
        card_texts("browser.drag", all, CardInputs::default())
    };
    assert_eq!(
        drag(json!({ "mode": "element", "to": "Won" })),
        ["“Deal card”", "Onto “Won”"]
    );
    assert_eq!(
        drag(json!({ "mode": "element" })),
        ["“Deal card”", "Pick where to drop it"]
    );
    assert_eq!(
        drag(json!({ "mode": "by", "dx": 300, "dy": -40 })),
        ["“Deal card”", "By 300, -40 px"]
    );
    assert_eq!(
        drag(json!({ "mode": "path", "path": "0, 0\n40, 40\n120, -80", "cluster": "Board" })),
        ["“Deal card”", "Along 3 points", "Zoom group “Board”"]
    );
    assert_eq!(
        drag(json!({ "mode": "path", "path": "" })),
        ["“Deal card”", "Add points to the path"]
    );
    assert_eq!(
        drag(json!({ "mode": "path", "path": "100, 0" })),
        ["“Deal card”", "Along 1 point"]
    );
    assert_eq!(
        drag(json!({ "mode": "path", "path": "100, 0\nnot a point" })),
        ["“Deal card”", "Line 2 of the path isn't two numbers"]
    );
    assert_eq!(
        drag(json!({ "mode": "path", "path": "0, 0" })),
        ["“Deal card”", "Add a point away from where the drag starts"]
    );
    let hover = |ms: f64| {
        card_texts(
            "browser.hover",
            json!({ "target": "Menu", "hoverMs": ms, "after": 0 }),
            CardInputs::default(),
        )
    };
    assert_eq!(hover(600.0), ["“Menu”", "Hover 0.6 s"]);
    assert_eq!(hover(0.0), ["“Menu”", "Passes over it"]);
    let mock = node_card(
        "browser.mockRoute",
        json!({ "url": " " }),
        CardInputs::default(),
    )[0]
    .clone();
    assert_eq!(
        (mock.text.as_str(), mock.tone),
        ("Add an address pattern", Some(Tone::Warn))
    );
    assert_eq!(
        parse_params("browser.mockRoute", json!({ "url": "" })).unwrap_err(),
        "Add an address pattern"
    );
    assert!(parse_params("browser.mockRoute", json!({})).is_ok());
    assert_eq!(
        node_card(
            "browser.setState",
            json!({ "script": " " }),
            CardInputs::default()
        )[0]
        .tone,
        Some(Tone::Warn)
    );
}

#[test]
fn says_on_the_click_card_when_it_double_clicks_or_right_clicks() {
    let first =
        |params: Value| card_texts("browser.click", params, CardInputs::default())[0].clone();
    assert_eq!(first(json!({ "target": "Reports" })), "“Reports”");
    assert_eq!(
        first(json!({ "target": "Reports", "double": true })),
        "Double-click “Reports”"
    );
    assert_eq!(
        first(json!({ "target": "Reports", "button": "right" })),
        "Right-click “Reports”"
    );
    assert_eq!(
        first(json!({ "target": "Reports", "button": "right", "double": true })),
        "Right-click “Reports”"
    );
    assert_eq!(
        first(json!({ "target": "", "double": true })),
        "Pick an element"
    );
}

#[test]
fn checks_browser_action_settings_before_a_run() {
    for kind in ["browser.wait", "browser.assert"] {
        assert!(problem(kind, json!({ "timeoutMs": 0 })).is_some(), "{kind}");
        assert_eq!(
            problem(
                kind,
                json!({ "mode": "network-idle", "text": "Saved", "timeoutMs": 500 })
            ),
            None,
            "{kind}"
        );
    }
    let upload = |file: &str| {
        problem(
            "browser.upload",
            json!({ "target": "Avatar", "file": file }),
        )
    };
    assert_eq!(
        [
            upload("assets/photo.png"),
            upload("project:assets/photo.png")
        ],
        [None, None]
    );
    assert_eq!(upload("").as_deref(), Some("Choose a file"));
    for outside in [
        "../secret.txt",
        "assets/../../secret.txt",
        "/etc/passwd",
        "C:\\Users\\me\\a.txt",
    ] {
        assert_eq!(
            upload(outside).as_deref(),
            Some("Choose a file inside the project folder"),
            "{outside}"
        );
    }
    assert_eq!(
        problem("browser.wait", json!({ "mode": "element", "target": " " })).as_deref(),
        Some("Pick what to wait for")
    );
    assert_eq!(
        problem("browser.wait", json!({ "mode": "url", "url": "" })).as_deref(),
        Some("Add an address pattern")
    );
    for ok in [
        json!({ "mode": "url", "url": "*/done*" }),
        json!({ "mode": "network-idle" }),
        json!({ "mode": "ms", "ms": 500 }),
    ] {
        assert_eq!(problem("browser.wait", ok.clone()), None, "{ok}");
    }
    let path = |p: &str| {
        problem(
            "browser.drag",
            json!({ "target": "Card", "mode": "path", "path": p }),
        )
    };
    assert_eq!(path("0, 0\n40, 40"), None);
    assert_eq!(
        path("100, 0\nnot a point").as_deref(),
        Some("Line 2 of the path isn't two numbers")
    );
    assert_eq!(path("").as_deref(), Some("Add points to the path"));
    assert_eq!(
        problem(
            "browser.drag",
            json!({ "target": "Card", "mode": "element", "to": "" })
        )
        .as_deref(),
        Some("Pick where to drop it")
    );
    assert_eq!(
        problem(
            "browser.drag",
            json!({ "target": "Card", "mode": "element", "to": "Done" })
        ),
        None
    );
    assert_eq!(
        problem("browser.select", json!({ "target": "Plan", "value": "" })).as_deref(),
        Some("Choose an option")
    );
    assert_eq!(
        problem(
            "browser.select",
            json!({ "target": "Plan", "value": "pro" })
        ),
        None
    );
    let typed = |text: &str| problem("browser.type", json!({ "target": "Name", "text": text }));
    assert_eq!(typed("").as_deref(), Some("Add text to type"));
    assert_eq!(
        typed(&format!("{PAUSE_MARK}{PAUSE_MARK}")).as_deref(),
        Some("Add text to type")
    );
    assert_eq!(typed(&format!("tom{PAUSE_MARK}smith")), None);
    assert_eq!(
        problem("browser.assert", json!({ "text": " " })).as_deref(),
        Some("Add the text to look for")
    );
}

#[test]
fn asks_a_new_action_only_for_what_the_user_has_to_fill_in() {
    let firsts: IndexMap<String, Option<String>> = nodes()
        .into_iter()
        .map(|n| (n.kind.to_string(), problem(&n.kind, json!({}))))
        .collect();
    let pick = Some("Pick an element".to_owned());
    let expected: IndexMap<String, Option<String>> = [
        ("browser.open", None),
        ("browser.signIn", None),
        ("browser.startRecording", None),
        ("browser.navigate", None),
        ("browser.click", pick.clone()),
        ("browser.type", pick.clone()),
        ("browser.hover", pick.clone()),
        ("browser.drag", pick.clone()),
        ("browser.scroll", pick.clone()),
        ("browser.select", pick.clone()),
        ("browser.upload", pick.clone()),
        ("browser.press", None),
        ("browser.wait", Some("Pick what to wait for".into())),
        ("browser.assert", Some("Add the text to look for".into())),
        ("browser.hoist", pick.clone()),
        ("browser.setState", None),
        ("browser.mockRoute", None),
        ("browser.switchTheme", None),
        ("browser.changeLanguage", None),
        ("browser.freezeClock", None),
        ("browser.hideElements", None),
        ("browser.chapter", None),
        ("browser.overlayHtml", None),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    assert_eq!(firsts, expected);
    assert_eq!(
        problem("browser.scroll", json!({ "mode": "element", "target": "" })),
        pick
    );
    assert_eq!(
        problem(
            "browser.scroll",
            json!({ "mode": "element", "target": "Pricing" })
        ),
        None
    );
    assert_eq!(
        problem("browser.scroll", json!({ "mode": "by", "target": "" })),
        None
    );
}

#[test]
fn keeps_browser_numbers_inside_their_limits_and_draws_every_card() {
    let limits = [
        (
            "browser.scroll",
            json!({ "mode": "by", "deltaY": 20_001 }),
            "Distance must be at most 20,000 px",
        ),
        (
            "browser.wait",
            json!({ "mode": "ms", "ms": 600_001 }),
            "Time must be at most 10 min",
        ),
        (
            "browser.drag",
            json!({ "target": "Card", "mode": "by", "dx": -10_001 }),
            "Across must be at least −10,000 px",
        ),
        (
            "browser.open",
            json!({ "viewportWidth": 10 }),
            "Window width must be at least 320 px",
        ),
        (
            "browser.click",
            json!({ "target": "Save", "after": -1 }),
            "Pause after must be at least 0 s",
        ),
    ];
    for (kind, params, message) in limits {
        assert_eq!(problem(kind, params).as_deref(), Some(message), "{kind}");
    }
    for n in nodes() {
        let rows = node_card(&n.kind, json!({}), CardInputs::default());
        assert!(
            !rows.is_empty() && rows.iter().all(|r| !r.text.trim().is_empty()),
            "{}",
            n.kind
        );
    }
}

#[test]
fn fills_variables_into_text_settings_before_a_run() {
    let fill = |kind: &str, params: Value, vars: &[(&str, &str)]| -> Map<String, Value> {
        let vars: Vars = vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        interpolated_params(node_def(kind), params.as_object().unwrap(), &vars)
    };
    assert_eq!(
        fill(
            "browser.navigate",
            json!({ "url": "{{Site address}}/pricing" }),
            &[("Site address", "https://example.com")]
        )["url"],
        "https://example.com/pricing"
    );
    let typed = fill(
        "browser.type",
        json!({ "target": "{{Field}}", "text": "Hello {{Name}}" }),
        &[("Field", "Name"), ("Name", "Ada")],
    );
    assert_eq!(
        (typed["target"].clone(), typed["text"].clone()),
        (json!("Name"), json!("Hello Ada"))
    );
}

fn take() -> Graph {
    let mut g = Graph::default();
    let ids = [
        g.node(
            "open",
            "browser.open",
            json!({ "viewportWidth": 1280, "viewportHeight": 720 }),
        ),
        g.node(
            "go",
            "browser.navigate",
            json!({ "url": "{{Site address}}/app" }),
        ),
        g.labelled(
            "click",
            "browser.click",
            json!({ "target": "Save" }),
            "Save it",
        ),
        g.node("rec", "capture.record", json!({})),
    ];
    g.chain(&ids);
    g
}

fn session_at(wf: &Workflow, id: &str) -> Option<BrowserSessionValue> {
    match compile_workflow(wf, None).values.get(id)?.get("session")? {
        CompileValue::BrowserSession(s) => Some(s.clone()),
        _ => None,
    }
}

fn step_ids(wf: &Workflow, id: &str) -> Vec<String> {
    session_at(wf, id)
        .unwrap()
        .steps
        .into_iter()
        .map(|s| s.id)
        .collect()
}

#[test]
fn lists_session_actions_in_chain_order_with_their_settings_filled_in() {
    let mut wf = take().build();
    wf.variables
        .insert("Site address".into(), "https://example.com".into());
    let session = session_at(&wf, "click").unwrap();
    assert_eq!(session.browser.node_id, "open");
    assert_eq!(
        (
            session.browser.params["viewportWidth"].as_f64(),
            session.browser.params["viewportHeight"].as_f64()
        ),
        (Some(1280.0), Some(720.0))
    );
    let steps: Vec<(String, String, Option<String>)> = session
        .steps
        .iter()
        .map(|s| (s.id.clone(), s.kind.kind().to_owned(), s.label.clone()))
        .collect();
    assert_eq!(
        steps,
        [
            ("go".into(), "browser.navigate".into(), None),
            (
                "click".into(),
                "browser.click".into(),
                Some("Save it".into())
            )
        ]
    );
    assert_eq!(session.steps[0].params["url"], "https://example.com/app");
}

#[test]
fn leaves_skipped_actions_out_and_gives_each_fork_its_own_list() {
    let mut wf = take().build();
    wf.nodes
        .iter_mut()
        .find(|n| n.id == "click")
        .unwrap()
        .disabled = Some(true);
    assert_eq!(step_ids(&wf, "click"), ["go"]);
    let mut g = take();
    g.node("other", "browser.press", json!({ "keys": "Escape" }));
    g.node("shot", "capture.still", json!({}));
    g.chain(&["go".into(), "other".into(), "shot".into()]);
    let wf = g.build();
    assert_eq!(step_ids(&wf, "click"), ["go", "click"]);
    assert_eq!(step_ids(&wf, "other"), ["go", "other"]);
    let mut g = Graph::default();
    let ids = [
        g.node(
            "go",
            "browser.navigate",
            json!({ "url": "https://example.com" }),
        ),
        g.node("click", "browser.click", json!({ "target": "Save" })),
    ];
    g.chain(&ids);
    assert!(session_at(&g.build(), "click").is_none());
}
