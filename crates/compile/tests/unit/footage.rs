use opsis_compile::footage::{element_footage, footage_uses};
use opsis_compile::hosted::hosted;
use opsis_types::timeline::{TimelineItem, TimelineSpec};
use proptest::prelude::*;
use serde_json::{Map, Value, json};

fn props(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

fn element(id: &str, component: &str, props: Value) -> Value {
    json!({ "id": id, "from": 0, "durationInFrames": 30, "kind": "element", "component": component, "props": props })
}

fn film(items: Vec<Value>) -> TimelineSpec {
    serde_json::from_value(json!({
        "width": 1920,
        "height": 1080,
        "fps": 30,
        "durationInFrames": 30,
        "tracks": [{ "id": "t", "items": items }],
        "audio": [],
        "assets": { "projectBase": "http://127.0.0.1:9/" },
    }))
    .unwrap()
}

#[test]
fn a_video_plays_as_footage_and_an_image_only_when_graded() {
    let video = element_footage(
        "video",
        &props(json!({ "src": "project:a.mp4", "loop": true, "trimBeforeMs": 500 })),
    );
    assert_eq!(video.len(), 1);
    assert_eq!(
        (
            video[0].prop.as_str(),
            video[0].used.trim_ms,
            video[0].used.looping
        ),
        ("src", 500, true)
    );
    assert!(element_footage("image", &props(json!({ "src": "project:a.png" }))).is_empty());
    let graded = props(
        json!({ "src": "project:a.png", "footage": { "grade": [{ "nodeId": "g", "type": "grade.mono", "params": {} }] } }),
    );
    assert_eq!(element_footage("image", &graded).len(), 1);
    assert!(element_footage("video", &props(json!({ "src": "" }))).is_empty());
    let looping = element_footage("video-loop", &props(json!({ "src": "project:b.mp4" })));
    assert!(looping[0].used.looping);
}

#[test]
fn a_file_setting_plays_as_footage_only_when_its_kind_is_video() {
    let hero = props(
        json!({ "image1": "project:a.mp4", "image1Kind": "video", "image2": "project:b.png", "image2Kind": "image" }),
    );
    let found = element_footage("hero-launch", &hero);
    assert_eq!(
        found.iter().map(|f| f.prop.as_str()).collect::<Vec<_>>(),
        ["image1"]
    );
    assert!(element_footage("title", &props(json!({ "src": "project:a.mp4" }))).is_empty());
}

proptest! {
    #[test]
    fn the_same_use_has_the_same_id_and_any_change_another(trim in 0_u32..10_000, other in 0_u32..10_000, looping: bool) {
        let one = |trim: u32, looping: bool| element_footage("video", &props(json!({ "src": "project:a.mp4", "loop": looping, "trimBeforeMs": trim })))[0].used.id.clone();
        prop_assert_eq!(one(trim, looping), one(trim, looping));
        prop_assert_eq!(one(trim, looping) == one(other, !looping), false);
        prop_assert_eq!(one(trim, looping) == one(other, looping), trim == other);
    }
}

#[test]
fn hosting_gives_every_use_its_footage_address_and_lists_each_once() {
    let spec = film(vec![
        element("a", "video", json!({ "src": "project:a.mp4" })),
        element("b", "video", json!({ "src": "project:a.mp4" })),
        element(
            "c",
            "laptop-frame",
            json!({ "screenSrc": "project:c.mp4", "screenKind": "video" }),
        ),
    ]);
    let uses = footage_uses(&spec.tracks);
    assert_eq!(uses.len(), 2);
    let out = hosted(&spec);
    let urls: Vec<Value> = out.tracks[0]
        .items
        .iter()
        .map(|item| match item {
            TimelineItem::Element { props, .. } => props["footageUrls"].clone(),
            _ => Value::Null,
        })
        .collect();
    let url = |id: &str| format!("http://127.0.0.1:9/.opsis/footage/{id}/");
    assert_eq!(
        urls,
        [
            json!({ "src": url(&uses[0].id) }),
            json!({ "src": url(&uses[0].id) }),
            json!({ "screenSrc": url(&uses[1].id) }),
        ]
    );
}
