use opsis_compile::graph::{
    IssueLevel, connection_problem, run_targets, topo_sort, validate_workflow,
};
use opsis_types::workflow::Workflow;
use serde_json::json;

use crate::helpers::{Graph, template};

fn todo_mvc() -> Workflow {
    serde_json::from_value(template("todomvc")).unwrap()
}

fn disable(wf: &mut Workflow, id: &str, disabled: bool) {
    wf.nodes.iter_mut().find(|n| n.id == id).unwrap().disabled = Some(disabled);
}

fn messages_for(wf: &Workflow, id: &str) -> Vec<String> {
    validate_workflow(wf, None)
        .into_iter()
        .filter(|i| i.node_id.as_deref() == Some(id))
        .map(|i| i.message)
        .collect()
}

#[test]
fn sorts_and_validates_the_sample() {
    let wf = todo_mvc();
    let sorted = topo_sort(&wf);
    let order: Vec<&str> = sorted.sorted.iter().map(|n| n.id.as_str()).collect();
    let at = |id: &str| order.iter().position(|o| *o == id).unwrap();
    assert!(sorted.cyclic.is_empty());
    assert!(
        at("session") < at("record")
            && at("record") < at("camera")
            && at("sequence") < at("render")
    );
    assert!(!order.contains(&"note"));
    assert!(
        validate_workflow(&wf, None)
            .iter()
            .all(|i| i.level != IssueLevel::Error)
    );
}

#[test]
fn checks_a_new_connection_the_same_way_for_the_canvas_and_the_api() {
    let wf = todo_mvc();
    assert_eq!(
        connection_problem(&wf, "camera", "clip", "camera", "clip")
            .unwrap()
            .message,
        "A node can't connect to itself."
    );
    assert!(
        connection_problem(&wf, "gone", "clip", "camera", "clip")
            .unwrap()
            .not_found
    );
    let missing = connection_problem(&wf, "camera", "clip", "record", "nope").unwrap();
    assert!(
        !missing.not_found && missing.message.contains("has no input “nope”"),
        "{}",
        missing.message
    );
    assert_eq!(
        connection_problem(&wf, "camera", "clip", "open", "session")
            .unwrap()
            .message,
        "A clip can't connect to a browser session input."
    );
}

#[test]
fn asks_for_the_browser_where_an_action_chain_doesnt_start_from_open_browser() {
    let mut g = Graph::default();
    let ids = [
        g.node(
            "go",
            "browser.navigate",
            json!({ "url": "https://example.com" }),
        ),
        g.node("rec", "capture.record", json!({})),
    ];
    g.node("lone", "capture.record", json!({}));
    g.chain(&ids);
    let errors: Vec<(String, String)> = validate_workflow(&g.build(), None)
        .into_iter()
        .filter(|i| i.level == IssueLevel::Error)
        .map(|i| (i.node_id.unwrap_or_default(), i.message))
        .collect();
    let expected = [
        ("go", "Connect something to “Browser”"),
        ("lone", "Connect something to “Browser”"),
    ]
    .map(|(a, b)| (a.to_owned(), b.to_owned()));
    assert_eq!(errors, expected);
}

#[test]
fn asks_for_another_action_when_a_sounds_action_is_gone() {
    let mut g = Graph::default();
    g.node("save", "browser.click", json!({ "target": "Save" }));
    g.node(
        "on",
        "audio.sfx",
        json!({ "src": "a.wav", "atAction": "save" }),
    );
    g.node(
        "gone",
        "audio.sfx",
        json!({ "src": "a.wav", "atAction": "deleted" }),
    );
    let ids: Vec<String> = validate_workflow(&g.build(), None)
        .into_iter()
        .filter(|i| i.message.contains("action this sound"))
        .filter_map(|i| i.node_id)
        .collect();
    assert_eq!(ids, ["gone"]);
}

#[test]
fn ignores_a_connection_from_a_deleted_node_when_counting_connections() {
    let mut g = Graph::default();
    g.node("rec", "capture.record", json!({}));
    g.link_ports("gone", "rec", "session", "session");
    let wf = g.build();
    let messages: Vec<String> = validate_workflow(&wf, None)
        .into_iter()
        .filter(|i| i.node_id.as_deref() == Some("rec") || i.edge_id.is_some())
        .map(|i| i.message)
        .collect();
    assert!(
        messages.contains(&"Connect something to “Browser”".to_owned()),
        "{messages:?}"
    );
    assert!(
        messages.contains(&"A connection points at a deleted node".to_owned()),
        "{messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|m| m.contains("only take one connection"))
    );
}

#[test]
fn warns_a_node_fed_by_a_skipped_node_that_has_nothing_to_pass_on() {
    let mut g = Graph::default();
    g.node("bg", "fx.core.solid", json!({}));
    g.node("trim", "timeline.trim", json!({}));
    g.node("speed", "timeline.speed", json!({}));
    g.link("bg", "trim").link("trim", "speed");
    let mut wf = g.build();
    let warned = |w: &Workflow| -> Vec<String> {
        validate_workflow(w, None)
            .into_iter()
            .filter(|i| i.message.contains("is skipped"))
            .filter_map(|i| i.node_id)
            .collect()
    };
    disable(&mut wf, "bg", true);
    assert_eq!(warned(&wf), ["trim"]);
    disable(&mut wf, "bg", false);
    disable(&mut wf, "trim", true);
    assert!(warned(&wf).is_empty());
}

#[test]
fn stops_two_nodes_from_saving_the_same_sign_in_or_file() {
    let mut g = Graph::default();
    g.node(
        "a",
        "browser.signIn",
        json!({ "url": "https://x.test/login", "secretName": "acct" }),
    );
    g.node(
        "b",
        "browser.signIn",
        json!({ "url": "https://x.test/login", "secretName": "acct" }),
    );
    g.node(
        "c",
        "browser.signIn",
        json!({ "url": "https://x.test/login", "secretName": "other" }),
    );
    let clashes: Vec<String> = validate_workflow(&g.build(), None)
        .into_iter()
        .filter(|i| i.message.contains("also saves the sign-in"))
        .filter_map(|i| i.node_id)
        .collect();
    assert_eq!(clashes, ["b"]);
    let mut g = Graph::default();
    g.node("one", "output.render", json!({ "fileName": "film" }));
    g.node("two", "output.render", json!({ "fileName": "film" }));
    g.node(
        "wide",
        "output.render",
        json!({ "fileName": "film", "variants": ["9:16"] }),
    );
    g.node("other", "output.render", json!({ "fileName": "teaser" }));
    let clashes: Vec<(String, String)> = validate_workflow(&g.build(), None)
        .into_iter()
        .filter(|i| i.message.contains("also saves"))
        .map(|i| (i.node_id.unwrap_or_default(), i.message))
        .collect();
    let message = "“Export video” also saves “film.mp4”. Give one of them another file name.";
    assert_eq!(
        clashes,
        [
            ("two".to_owned(), message.to_owned()),
            ("wide".to_owned(), message.to_owned())
        ]
    );
}

#[test]
fn names_a_variable_the_project_doesnt_have() {
    let mut g = Graph::default();
    let ids = [
        g.node("session", "browser.open", json!({})),
        g.node(
            "go",
            "browser.navigate",
            json!({ "url": "{{Site address}}/pricing" }),
        ),
        g.node("rec", "capture.record", json!({})),
    ];
    g.chain(&ids);
    let mut wf = g.build();
    assert_eq!(
        messages_for(&wf, "go"),
        [
            "“Site address” isn't one of the project's variables. Add it under Variables in the project settings, or change this setting."
        ]
    );
    wf.variables
        .insert("Site address".into(), "https://example.com".into());
    assert!(messages_for(&wf, "go").is_empty());
}

#[test]
fn warns_that_a_reload_before_the_first_page_does_nothing() {
    let reloads = |before_page: bool| {
        let mut g = Graph::default();
        let session = g.node("session", "browser.open", json!({}));
        let script = g.node(
            "script",
            "browser.setState",
            json!({ "script": "1", "reload": true }),
        );
        let go = g.node(
            "go",
            "browser.navigate",
            json!({ "url": "https://example.com" }),
        );
        let rec = g.node("rec", "capture.record", json!({}));
        let order = if before_page {
            [session, script, go, rec]
        } else {
            [session, go, script, rec]
        };
        g.chain(&order);
        messages_for(&g.build(), "script")
    };
    let before = reloads(true);
    assert!(
        before.len() == 1 && before[0].contains("there's nothing to reload"),
        "{before:?}"
    );
    assert!(reloads(false).is_empty());
}

#[test]
fn the_picker_opens_the_last_page_the_session_went_to_or_the_site_address() {
    use opsis_compile::compile::compile_workflow;
    use opsis_compile::inspector::{PickerTarget, inspector_context};
    use opsis_compile::vars::SITE_ADDRESS_VAR;
    let picker = |pages: &[&str], site: Option<&str>| {
        let mut g = Graph::default();
        let mut order = vec![g.node("session", "browser.open", json!({}))];
        for (i, url) in pages.iter().enumerate() {
            order.push(g.node(&format!("go{i}"), "browser.navigate", json!({ "url": url })));
        }
        order.push(g.node("click", "browser.click", json!({})));
        g.chain(&order);
        let mut wf = g.build();
        if let Some(site) = site {
            wf.variables
                .insert(SITE_ADDRESS_VAR.to_owned(), site.to_owned());
        }
        let compiled = compile_workflow(&wf, None);
        match inspector_context(&wf, &compiled, &indexmap::IndexMap::new(), "click")
            .unwrap()
            .picker
        {
            PickerTarget::Page(page) => Some(page.url),
            PickerTarget::Problem { .. } => None,
        }
    };
    assert_eq!(
        picker(
            &["https://a.test", "https://b.test"],
            Some("https://site.test")
        ),
        Some("https://b.test".to_owned())
    );
    assert_eq!(
        picker(&[], Some("https://site.test")),
        Some("https://site.test".to_owned())
    );
    assert_eq!(picker(&[], None), None);
}

#[test]
fn reports_group_and_note_settings_that_cant_be_read() {
    let mut g = Graph::default();
    g.node("note", "core.note", json!({ "text": 5 }));
    g.node("group", "core.group", json!({}));
    let ids: Vec<String> = validate_workflow(&g.build(), None)
        .into_iter()
        .filter_map(|i| i.node_id)
        .collect();
    assert_eq!(ids, ["note"]);
}

#[test]
fn rejects_port_type_mismatches_and_cycles() {
    let mut g = Graph::default();
    g.node("a", "timeline.loop", json!({}));
    g.node("b", "timeline.loop", json!({}));
    g.link("a", "b").link("b", "a");
    g.node("rec", "capture.record", json!({}));
    g.link_ports("a", "rec", "clip", "session");
    let issues = validate_workflow(&g.build(), None);
    assert!(issues.iter().any(|i| i.message.contains("loops back")));
    assert!(
        issues
            .iter()
            .any(|i| i.message == "A clip can't connect to a browser session input")
    );
}

fn sorted(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids
}

#[test]
fn runs_every_export_and_the_nodes_after_one_or_everything_without_an_export() {
    let mut g = Graph::default();
    g.node("title", "fx.core.title", json!({}));
    g.node("film", "output.render", json!({}));
    g.link("title", "film");
    g.node("gif", "media.gif", json!({}));
    g.link_ports("film", "gif", "media", "media");
    g.node("web", "media.faststart", json!({}));
    g.link_ports("gif", "web", "media", "media");
    g.node("loose", "media.import", json!({ "path": "a.mp4" }));
    g.node("tidy", "media.faststart", json!({}));
    let mut wf = g.build();
    assert_eq!(sorted(run_targets(&wf)), ["film", "gif", "web"]);
    disable(&mut wf, "gif", true);
    assert_eq!(sorted(run_targets(&wf)), ["film", "web"]);
    let mut g = Graph::default();
    g.node("a", "media.import", json!({ "path": "a.mp4" }));
    g.node("b", "media.faststart", json!({}));
    assert_eq!(sorted(run_targets(&g.build())), ["a", "b"]);
    let mut g = Graph::default();
    let ids = [
        g.node("session", "browser.open", json!({})),
        g.node("login", "browser.signIn", json!({ "secretName": "admin" })),
        g.node(
            "go",
            "browser.navigate",
            json!({ "url": "https://example.com" }),
        ),
        g.node("rec", "capture.record", json!({})),
    ];
    g.chain(&ids);
    assert_eq!(run_targets(&g.build()), ["login", "rec"]);
}
