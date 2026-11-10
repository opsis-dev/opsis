use indexmap::IndexMap;
use opsis_compile::compile::compile_workflow;
use opsis_compile::node::CardInputs;
use opsis_types::numbers;
use opsis_types::timeline::AudioItem;
use opsis_types::values::{CompileValue, NodeResult};
use opsis_types::workflow::{Marker, Workflow};
use serde_json::{Value, json};

use crate::helpers::{Graph, TITLE, action, card_texts, make_log, node_result, record_result};

fn riser_result() -> NodeResult {
    node_result(json!({
        "outputs": { "audio": {
            "type": "media", "kind": "audio", "src": "project:.opsis/cache/r/made.wav",
            "durationMs": 3000, "hasAudio": true
        } },
        "artifacts": [
            { "kind": "audio", "path": ".opsis/cache/r/made.wav" },
            { "kind": "audio", "path": ".opsis/cache/r/end-hit.wav" }
        ]
    }))
}

fn riser(params: Value) -> Value {
    let mut all = json!({ "source": "made", "sound": "riser", "timing": "ends-on-action" });
    if let (Some(all), Value::Object(own)) = (all.as_object_mut(), params) {
        all.extend(own);
    }
    all
}

fn mixed(frames: u32, params: Value, markers: Vec<Marker>) -> Vec<AudioItem> {
    let mut g = Graph::default();
    g.node("title", TITLE, json!({ "clipDuration": frames }));
    g.node("rise", "audio.sfx", riser(params));
    g.node("mix", "audio.mix", json!({}));
    g.link("title", "mix")
        .link_ports("rise", "mix", "audio", "sounds");
    let mut wf: Workflow = g.build();
    wf.markers = markers;
    let results: IndexMap<String, NodeResult> =
        IndexMap::from([("rise".to_owned(), riser_result())]);
    match compile_workflow(&wf, Some(&results)).values["mix"]["clip"].clone() {
        CompileValue::Clip(clip) => clip.audio,
        other => panic!("{other:?}"),
    }
}

fn ends(item: &AudioItem) -> i32 {
    item.from + i32::try_from(item.length_frames.unwrap_or(0)).unwrap()
        - i32::try_from(item.trim_before.unwrap_or(0)).unwrap()
}

#[test]
fn ends_on_the_end_of_its_clip_and_trims_its_start_when_the_clip_is_shorter() {
    let long = mixed(150, json!({ "timing": "ends-with-clip" }), Vec::new());
    assert_eq!(
        (long[0].from, long[0].trim_before, ends(&long[0])),
        (60, None, 150)
    );
    let short = mixed(60, json!({ "timing": "ends-with-clip" }), Vec::new());
    assert_eq!(
        (short[0].from, short[0].trim_before, ends(&short[0])),
        (0, Some(30), 60)
    );
}

#[test]
fn ends_on_a_marker_with_a_hit_landing_on_it() {
    let marker = Marker {
        id: "launch".to_owned(),
        at_ms: 4000.0,
        label: "Launch".to_owned(),
    };
    let items = mixed(
        200,
        json!({ "timing": "ends-on-marker", "marker": "launch", "endHit": "pop" }),
        vec![marker],
    );
    assert_eq!(items.len(), 2);
    assert_eq!(
        (items[0].src.as_str(), ends(&items[0])),
        ("project:.opsis/cache/r/made.wav", 120)
    );
    assert_eq!(
        (items[1].src.as_str(), items[1].from),
        ("project:.opsis/cache/r/end-hit.wav", 120)
    );
    assert_eq!(
        mixed(
            200,
            json!({ "timing": "ends-on-marker", "marker": "gone" }),
            Vec::new()
        ),
        []
    );
}

#[test]
fn says_what_it_makes_and_what_it_still_needs() {
    assert_eq!(
        card_texts("audio.sfx", riser(json!({})), CardInputs::default()),
        [
            "Riser · default · 3 s",
            "Pick the action it ends on",
            "Made when the node runs"
        ]
    );
    assert_eq!(
        card_texts(
            "audio.sfx",
            riser(json!({ "timing": "ends-with-clip", "variation": "custom", "customMs": 1500 })),
            CardInputs::default()
        )[..2],
        ["Riser · custom · 1.5 s", "Ends with the clip"]
    );
}

#[test]
fn ends_on_a_browser_action_even_when_that_is_sooner_than_its_length() {
    let mut g = Graph::default();
    let cam = g.recorded_take(&[]);
    g.node("rise", "audio.sfx", riser(json!({ "atAction": "launch" })));
    g.node("mix", "audio.mix", json!({}));
    g.link(&cam, "mix")
        .link_ports("rise", "mix", "audio", "sounds");
    let wf = g.build();
    let point = json!({ "x": 400, "y": 300 });
    let log = make_log(
        vec![action(
            json!({ "id": "c", "kind": "click", "t": 2000, "tDown": 2000, "point": point, "nodeId": "launch" }),
        )],
        6000.0,
    );
    let results: IndexMap<String, NodeResult> = [
        ("record".to_owned(), record_result(&log)),
        ("rise".to_owned(), riser_result()),
    ]
    .into_iter()
    .collect();
    let compiled = compile_workflow(&wf, Some(&results));
    let CompileValue::Clip(clip) = compiled.values["mix"]["clip"].clone() else {
        panic!("no clip");
    };
    let flat = opsis_compile::timeline::flatten::flatten_spec(&opsis_compile::clip::clip_to_spec(
        &clip,
        &wf.settings,
    ));
    let mark = flat
        .marks
        .iter()
        .find(|m| m.node_id.as_deref() == Some("launch"))
        .expect("the click's mark");
    let riser = &clip.audio[0];
    assert_eq!(ends(riser), numbers::whole_signed(mark.frame));
    assert!(
        riser.trim_before.is_some(),
        "a 3 s riser is trimmed to end on a click 2 s in"
    );
}

#[test]
fn names_the_action_or_marker_it_rises_into_on_the_card() {
    let mut g = Graph::default();
    let cam = g.recorded_take(&[("save", "browser.click", json!({ "target": "Save" }))]);
    g.labelled(
        "rise",
        "audio.sfx",
        riser(json!({ "atAction": "save" })),
        "Build-up",
    );
    g.node(
        "marked",
        "audio.sfx",
        riser(json!({ "timing": "ends-on-marker", "marker": "launch" })),
    );
    g.node(
        "beep",
        "audio.sfx",
        json!({ "source": "made", "sound": "beep", "timing": "on-action", "atAction": "save" }),
    );
    g.link(&cam, "rise");
    let mut wf: Workflow = g.build();
    wf.markers = vec![Marker {
        id: "launch".to_owned(),
        at_ms: 2000.0,
        label: "Launch".to_owned(),
    }];
    let compiled = compile_workflow(&wf, None);
    let cards = opsis_compile::cards::workflow_cards(
        &wf,
        &opsis_compile::cards::CardSources {
            compiled: &compiled,
            results: None,
            secrets: None,
            gone: &|_| false,
        },
    );
    let save = opsis_compile::registry::node_name(wf.node("save").unwrap());
    assert_eq!(cards["rise"][1].text, format!("Ends on “{save}”"));
    assert_eq!(cards["marked"][1].text, "Ends on “Launch”");
    assert_eq!(cards["beep"][1].text, format!("On “{save}”"));
}
