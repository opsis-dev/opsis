use opsis_render::picture::{PictureOptions, picture_key};
use opsis_types::timeline::TimelineSpec;
use serde_json::{Value, json};

fn spec(extra: Value) -> TimelineSpec {
    let mut spec = json!({
        "width": 1920, "height": 1080, "fps": 30, "durationInFrames": 300,
        "tracks": [{ "id": "t", "items": [{ "id": "i", "from": 0, "durationInFrames": 300, "kind": "element", "component": "core/title", "props": { "title": "Hi" } }] }],
        "audio": [{ "id": "a", "src": "app:sfx/pop.wav", "from": 10 }]
    });
    spec.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(spec).unwrap()
}

fn opts(extra: Value) -> PictureOptions {
    let mut opts = json!({ "codec": "h264", "crf": 18, "scale": 1, "gl": "angle" });
    opts.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(opts).unwrap()
}

#[test]
fn pictures_are_reused_when_only_the_sound_or_the_asset_server_changes() {
    let key = picture_key(&spec(json!({})), &opts(json!({})), "");
    assert_eq!(
        picture_key(&spec(json!({ "audio": [] })), &opts(json!({})), ""),
        key
    );
    assert_eq!(
        picture_key(
            &spec(json!({ "mix": { "src": "project:m.wav", "hash": "x" } })),
            &opts(json!({})),
            ""
        ),
        key
    );
    assert_eq!(
        picture_key(
            &spec(json!({ "assets": { "projectBase": "http://127.0.0.1:5/" } })),
            &opts(json!({})),
            ""
        ),
        key
    );
}

#[test]
fn pictures_render_again_when_the_picture_shape_or_quality_changes() {
    let key = picture_key(&spec(json!({})), &opts(json!({})), "");
    assert_ne!(
        picture_key(
            &spec(json!({ "width": 1080, "height": 1920 })),
            &opts(json!({})),
            ""
        ),
        key
    );
    assert_ne!(
        picture_key(&spec(json!({ "reframe": "center" })), &opts(json!({})), ""),
        key
    );
    assert_ne!(
        picture_key(&spec(json!({})), &opts(json!({ "crf": 23 })), ""),
        key
    );
}
