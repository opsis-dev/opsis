use indexmap::IndexMap;
use opsis_compile::clip::{empty_clip, scene_item_count};
use opsis_compile::limits::MAX_FRAMES;
use opsis_compile::node::CardInputs;
use opsis_compile::nodes::timeline::branch_port;
use opsis_compile::vars::Vars;
use opsis_types::cards::Tone;
use opsis_types::timeline::TransitionRef;
use opsis_types::values::{BrowserAction, ClipValue, CompileValue, PortValues};
use serde_json::{Map, Value, json};

use crate::helpers::{
    CLIP, action, card_texts, clip_value, compile_clip, compile_node_in, compile_node_with,
    group_value, make_log, node_card, node_card_with, node_result, parse_params, record_result,
};

#[test]
fn adds_the_takes_length_and_actions_and_says_where_the_video_starts() {
    let record = record_result(&make_log(
        vec![
            action(json!({ "id": "a", "kind": "click", "t": 1000 })),
            action(json!({ "id": "b", "kind": "click", "t": 2000 })),
        ],
        14_500.0,
    ));
    let texts: Vec<String> = node_card_with(
        "capture.record",
        json!({}),
        CardInputs::default(),
        Some(&record),
    )
    .into_iter()
    .map(|r| r.text)
    .collect();
    assert_eq!(texts, ["30 fps", "Smooth cursor", "14.5 s · 2 actions"]);
    let starts = |steps: &[&str]| {
        let inputs = CardInputs {
            steps: IndexMap::from([(
                "session".to_owned(),
                steps
                    .iter()
                    .map(|s| BrowserAction::of_kind(s).expect("a browser action"))
                    .collect(),
            )]),
            ..CardInputs::default()
        };
        card_texts("capture.record", json!({}), inputs)
    };
    assert!(
        starts(&["browser.navigate", "browser.click"])
            .contains(&"Starts with the first action".to_owned())
    );
    assert!(
        !starts(&["browser.navigate", "browser.startRecording"])
            .contains(&"Starts with the first action".to_owned())
    );
}

#[test]
fn tells_whether_a_sign_in_is_saved() {
    let status = |name: &str, secrets: &[&str]| {
        let inputs = CardInputs {
            secrets: Some(secrets.iter().map(|s| (*s).to_owned()).collect()),
            ..CardInputs::default()
        };
        node_card("browser.signIn", json!({ "secretName": name }), inputs)
            .into_iter()
            .find(|r| {
                serde_json::to_value(r.icon)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .starts_with("shield")
            })
            .unwrap()
    };
    assert_eq!(status("sauce demo", &["sauce-demo"]).text, "Signed in");
    let missing = status("other", &[]);
    assert_eq!(
        (missing.text.as_str(), missing.tone),
        ("Not signed in yet", Some(Tone::Warn))
    );
}

#[test]
fn describes_a_screenshot_and_shows_its_size_only_when_both_sides_are_known() {
    let first =
        |params: Value| node_card("capture.still", params, CardInputs::default())[0].clone();
    assert_eq!(first(json!({ "region": "full-page" })).text, "Whole page");
    assert_eq!(
        first(json!({ "region": "element", "target": "Pricing", "targetRole": "heading" })).text,
        "“Pricing” heading"
    );
    let empty = first(json!({ "region": "element", "target": "" }));
    assert_eq!(
        (empty.text.as_str(), empty.tone),
        ("Pick an element", Some(Tone::Warn))
    );
    let sized = |image: Value| {
        let mut img = json!({ "type": "image", "src": null });
        img.as_object_mut()
            .unwrap()
            .extend(image.as_object().unwrap().clone());
        let result = node_result(json!({ "outputs": { "image": img } }));
        node_card_with(
            "capture.still",
            json!({}),
            CardInputs::default(),
            Some(&result),
        )
        .into_iter()
        .map(|r| r.text)
        .collect::<Vec<_>>()
    };
    assert!(sized(json!({ "width": 1200, "height": 800 })).contains(&"1200 × 800".to_owned()));
    assert!(
        !sized(json!({ "width": 1200 }))
            .iter()
            .any(|t| t.contains('×'))
    );
}

#[test]
fn checks_screenshot_and_sign_in_settings() {
    assert_eq!(
        parse_params(
            "capture.still",
            json!({ "region": "element", "target": "" })
        )
        .unwrap_err(),
        "Pick an element"
    );
    assert!(
        parse_params(
            "capture.still",
            json!({ "region": "element", "target": "Go" })
        )
        .is_ok()
    );
    assert!(parse_params("capture.still", json!({ "region": "viewport" })).is_ok());
    let problem = |params: Value| parse_params("browser.signIn", params).err();
    assert_eq!(
        problem(json!({ "url": " " })).as_deref(),
        Some("Add the sign-in page")
    );
    assert_eq!(
        problem(
            json!({ "url": "https://example.com/login", "doneWhen": "selector", "selector": "" })
        )
        .as_deref(),
        Some("Add the element that appears after signing in")
    );
    assert_eq!(
        problem(
            json!({ "url": "https://example.com/login", "doneWhen": "selector", "selector": ".avatar" })
        ),
        None
    );
    let def = opsis_compile::registry::node_def("browser.signIn").unwrap();
    let vars = Vars::from([(
        "Site address".to_owned(),
        "https://app.example.com".to_owned(),
    )]);
    let interpolated = opsis_compile::compile::interpolated_params(Some(def), &Map::new(), &vars);
    assert_eq!(interpolated["url"], "https://app.example.com/login");
}

fn still(image: Option<Value>) -> PortValues {
    let result = image.map(|i| {
        let mut img = json!({ "type": "image" });
        img.as_object_mut()
            .unwrap()
            .extend(i.as_object().unwrap().clone());
        node_result(json!({ "outputs": { "image": img } }))
    });
    compile_node_with("capture.still", json!({}), json!({}), result.as_ref()).unwrap()
}

#[test]
fn gives_only_the_screenshot_once_it_is_taken() {
    let before = still(None);
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        json!({ "image": { "type": "image", "src": null } })
    );
    let after = still(Some(
        json!({ "src": "project:shot.png", "width": 400, "height": 300, "scale": 2 }),
    ));
    assert_eq!(
        serde_json::to_value(&after["image"]).unwrap(),
        json!({ "type": "image", "src": "project:shot.png", "width": 400, "height": 300, "scale": 2.0 })
    );
    assert!(!after.contains_key("clip"));
}

fn over(kind: &str, params: Value) -> ClipValue {
    compile_clip(kind, params, json!({ CLIP: clip_value(&empty_clip(90)) }))
}

#[test]
fn rounds_delays_and_trim_starts_to_whole_frames() {
    let start = |ms: u32| {
        over("timeline.delay", json!({ "ms": ms })).tracks[0].items[0]
            .base()
            .from
    };
    assert_eq!([start(10), start(0), start(500)], [0, 0, 15]);
    let from = |ms: u32| {
        group_value(&over("timeline.trim", json!({ "startMs": ms })))["startFrom"].as_f64()
    };
    assert_eq!([from(10), from(1000)], [Some(0.0), Some(30.0)]);
}

fn transition(kind: &str, frames: u32, hold: bool) -> TransitionRef {
    let mut t = json!({ "kind": kind, "durationInFrames": frames });
    if hold {
        t["hold"] = json!(true);
    }
    serde_json::from_value(t).unwrap()
}

#[test]
fn names_each_join_of_a_sequence_and_warns_when_one_covers_a_clip() {
    let context = |own: TransitionRef| CardInputs {
        connected: vec![
            "in1".into(),
            "in2".into(),
            "in3".into(),
            "transition".into(),
        ],
        frames: [
            ("in1".to_owned(), 300),
            ("in2".to_owned(), 100),
            ("in3".to_owned(), 300),
        ]
        .into_iter()
        .collect(),
        enters: [
            ("in3".to_owned(), own),
            ("transition".to_owned(), transition("fade", 10, false)),
        ]
        .into_iter()
        .collect(),
        clip_frames: Some(690),
        ..CardInputs::default()
    };
    assert_eq!(
        card_texts(
            "timeline.sequence",
            json!({}),
            context(transition("wipe", 50, false))
        ),
        [
            "3 clips · 23 s",
            "Fade, Wipe",
            "Covers 50% of a 3.33 s clip"
        ]
    );
    assert!(
        !card_texts(
            "timeline.sequence",
            json!({}),
            context(transition("wipe", 50, true))
        )
        .iter()
        .any(|t| t.starts_with("Covers"))
    );
    assert_eq!(
        card_texts(
            "timeline.sequence",
            json!({}),
            CardInputs {
                connected: vec!["in1".into(), "in2".into(), "in3".into()],
                ..CardInputs::default()
            }
        ),
        ["3 clips", "2 joins cut"]
    );
}

#[test]
fn marks_layers_for_a_sequencer_unless_they_keep_their_layout() {
    let stacked = |params: Value| {
        compile_clip(
            "timeline.stack",
            params,
            json!({ "in1": clip_value(&empty_clip(30)), "in2": clip_value(&empty_clip(30)) }),
        )
    };
    assert_eq!(scene_item_count(&stacked(json!({}))), 2);
    assert_eq!(scene_item_count(&stacked(json!({ "keepLayout": true }))), 1);
}

#[test]
fn refuses_a_repeat_longer_than_the_frame_limit() {
    let repeat = |times: u32| {
        compile_node_with(
            "timeline.loop",
            json!({ "times": times }),
            json!({ CLIP: clip_value(&empty_clip(MAX_FRAMES / 50)) }),
            None,
        )
    };
    assert!(
        repeat(100)
            .unwrap_err()
            .contains("Repeated 100 times the clip would run")
    );
    match repeat(50).unwrap().shift_remove("clip") {
        Some(CompileValue::Clip(c)) => assert_eq!(c.duration_in_frames, MAX_FRAMES),
        other => panic!("{other:?}"),
    }
}

#[test]
fn chooses_by_variable() {
    let cases = [
        (Some("A"), "A", "a"),
        (Some(" A "), "A", "a"),
        (Some("B"), "A", "b"),
        (Some("B"), "", "a"),
        (None, "", "b"),
        (Some("  "), "", "b"),
        (None, "A", "b"),
    ];
    for (value, equals, port) in cases {
        assert_eq!(branch_port(value, equals), port, "{value:?} {equals}");
    }
    let vars = Vars::from([("Version".to_owned(), "A".to_owned())]);
    let picked = compile_node_in(
        "timeline.branch",
        json!({ "variable": " Version ", "equals": "A" }),
        json!({ "a": clip_value(&empty_clip(10)), "b": clip_value(&empty_clip(20)) }),
        None,
        &vars,
    )
    .unwrap();
    assert!(matches!(&picked["clip"], CompileValue::Clip(c) if c.duration_in_frames == 10));
    let missing = compile_node_with(
        "timeline.branch",
        json!({ "variable": "Version", "equals": "" }),
        json!({ "a": clip_value(&empty_clip(10)) }),
        None,
    );
    assert_eq!(missing.unwrap_err(), "Connect a clip to “Otherwise”.");
    assert_eq!(
        card_texts(
            "timeline.branch",
            json!({ "equals": "" }),
            CardInputs::default()
        )[0],
        "Version has a value"
    );
    assert_eq!(
        card_texts(
            "timeline.branch",
            json!({ "equals": "A" }),
            CardInputs::default()
        )[0],
        "Version = “A”"
    );
}

#[test]
fn refuses_a_trim_past_the_end_and_warns_when_it_keeps_more_than_the_clip_has() {
    let trimmed = |ms: f64| {
        compile_node_with(
            "timeline.trim",
            json!({ "startMs": ms }),
            json!({ CLIP: clip_value(&empty_clip(60)) }),
            None,
        )
    };
    assert_eq!(
        trimmed(2000.0).unwrap_err(),
        "Start is at or after the end of the clip, which is 2 s long."
    );
    assert!(
        matches!(&trimmed(1000.0).unwrap()["clip"], CompileValue::Clip(c) if c.duration_in_frames == 30)
    );
    let texts = |p: Value| {
        card_texts(
            "timeline.trim",
            p,
            CardInputs {
                frames: IndexMap::from([("clip".to_owned(), 60)]),
                ..CardInputs::default()
            },
        )
    };
    assert!(
        texts(json!({ "startMs": 1000, "durationMs": 10_000 }))
            .contains(&"The clip ends at 2 s".to_owned())
    );
    assert!(
        !texts(json!({ "startMs": 500, "durationMs": 1000 }))
            .contains(&"The clip ends at 2 s".to_owned())
    );
}

#[test]
fn fits_lengths_by_holding_cutting_or_speeding() {
    let fit = |params: Value, frames: u32, audio: Option<Value>| {
        let mut inputs = json!({ CLIP: clip_value(&empty_clip(frames)) });
        if let Some(a) = audio {
            inputs["audio"] = a;
        }
        compile_clip("timeline.durationFit", params, inputs)
    };
    assert_eq!(
        group_value(&fit(json!({ "targetMs": 3000, "mode": "hold" }), 1, None))["remap"],
        json!([[0.0, 0.0], [90.0, 0.0]])
    );
    let texts = card_texts(
        "timeline.durationFit",
        json!({ "targetMs": 3000, "mode": "cut" }),
        CardInputs {
            frames: IndexMap::from([("clip".to_owned(), 30)]),
            clip_frames: Some(90),
            ..CardInputs::default()
        },
    );
    assert!(
        texts.contains(&"Ends 2 s early, then shows nothing".to_owned()),
        "{texts:?}"
    );
    let clip = fit(
        json!({ "targetMs": 2000, "mode": "speed" }),
        30,
        Some(json!({ "type": "audio", "items": [], "durationInFrames": 0 })),
    );
    assert_eq!(clip.duration_in_frames, 60);
    let trim_end = clip.tracks[0].items[0]
        .base()
        .handles
        .as_ref()
        .and_then(|h| h.trim_end.as_ref())
        .unwrap();
    assert_eq!(trim_end.param, "targetMs");
}
