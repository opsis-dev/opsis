use indexmap::IndexMap;
use opsis_compile::cards::{CardSources, workflow_cards};
use opsis_compile::compile::compile_workflow;
use opsis_compile::graph::{IssueLevel, validate_workflow};
use opsis_compile::node::CardInputs;
use opsis_types::values::{CompileValue, MediaKind, NodeResult};
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};

use crate::helpers::{
    Graph, compile_node_with, make_log, node_card_with, node_result, record_result,
};

fn checked(src: &str, media: Value, measured: Value) -> NodeResult {
    let mut media = media;
    media["type"] = json!("media");
    media["src"] = json!(src);
    let mut fields = json!({ "outputs": {} });
    fields["outputs"]["media"] = media;
    fields["measured"] = measured;
    node_result(fields)
}

fn video_result() -> NodeResult {
    checked(
        "project:assets/a.mp4",
        json!({ "kind": "video", "durationMs": 12_000, "width": 1920, "height": 1080, "hasAudio": false }),
        json!([{ "type": "file", "bytes": 2_621_440, "fps": 29.97 }]),
    )
}

fn texts(params: Value, inputs: CardInputs, result: Option<&NodeResult>) -> Vec<String> {
    node_card_with("media.import", params, inputs, result)
        .into_iter()
        .map(|r| r.text)
        .collect()
}

fn warnings(
    wf: &Workflow,
    results: Option<&IndexMap<String, NodeResult>>,
) -> Vec<(String, String)> {
    validate_workflow(wf, Some(&compile_workflow(wf, results)))
        .into_iter()
        .filter(|i| i.level == IssueLevel::Warning)
        .map(|i| (i.node_id.unwrap_or_default(), i.message))
        .collect()
}

#[test]
fn uses_a_checked_file_only_while_the_path_still_names_it() {
    let result = video_result();
    let media = |path: &str| match compile_node_with(
        "media.import",
        json!({ "path": path }),
        json!({}),
        Some(&result),
    )
    .unwrap()
    .shift_remove("media")
    {
        Some(CompileValue::Media(m)) => m,
        other => panic!("{other:?}"),
    };
    let same = media("assets/a.mp4");
    assert_eq!(
        (same.kind, same.duration_ms, same.has_audio),
        (MediaKind::Video, Some(12_000), Some(false))
    );
    let other = media("assets/b.png");
    assert_eq!(
        (other.src.as_deref(), other.kind, other.duration_ms),
        (Some("project:assets/b.png"), MediaKind::Image, None)
    );
}

#[test]
fn shows_what_the_check_read_and_says_when_a_file_is_missing_or_not_read_yet() {
    let result = video_result();
    assert_eq!(
        texts(
            json!({ "path": "assets/a.mp4" }),
            CardInputs::default(),
            Some(&result)
        ),
        [
            "a.mp4",
            "MP4 · 1920 × 1080 · 29.97 fps",
            "12 s",
            "2.5 MiB · No sound"
        ]
    );
    let sound = checked(
        "project:assets/voice.mp3",
        json!({ "kind": "audio", "durationMs": 3_000, "hasAudio": true }),
        json!([{ "type": "file", "bytes": 49_152, "fps": null }]),
    );
    assert_eq!(
        texts(
            json!({ "path": "assets/voice.mp3" }),
            CardInputs::default(),
            Some(&sound)
        ),
        ["voice.mp3", "MP3 · Sound", "3 s", "48 KiB"]
    );
    assert_eq!(
        texts(
            json!({ "path": "assets/b.mp4" }),
            CardInputs::default(),
            Some(&result)
        ),
        ["b.mp4", "Not read yet"]
    );
    assert_eq!(
        texts(
            json!({ "path": "assets/shot.png" }),
            CardInputs::default(),
            None
        ),
        ["shot.png", "Not read yet"]
    );
    let gone = CardInputs {
        missing_files: vec!["assets/a.mp4".to_owned()],
        ..CardInputs::default()
    };
    assert_eq!(
        texts(json!({ "path": "assets/a.mp4" }), gone, Some(&result)),
        ["a.mp4", "Not in the project folder"]
    );
}

#[test]
fn warns_when_a_connected_file_is_the_wrong_kind_before_anything_runs() {
    let mut g = Graph::default();
    g.node(
        "voice",
        "media.import",
        json!({ "path": "assets/voice.mp3" }),
    );
    g.node("shot", "media.import", json!({ "path": "assets/shot.png" }));
    g.node("resize", "image.resize", json!({}));
    g.node("picture", "media.soundPicture", json!({}));
    g.node("frame", "media.thumbnail", json!({}));
    g.node("pauses", "media.shortenPauses", json!({}));
    g.node("trim", "media.trim", json!({}));
    g.link_ports("voice", "resize", "media", "image");
    g.link_ports("shot", "picture", "media", "media");
    g.link_ports("voice", "frame", "media", "media");
    g.link_ports("voice", "pauses", "media", "media");
    g.link_ports("shot", "trim", "media", "media");
    let mut found = warnings(&g.build(), None);
    found.sort();
    assert_eq!(
        found,
        [
            (
                "frame".to_owned(),
                "The connected file is only sound, with no picture.".to_owned()
            ),
            (
                "picture".to_owned(),
                "The connected file has no sound.".to_owned()
            ),
            (
                "resize".to_owned(),
                "The connected file isn't an image.".to_owned()
            ),
            (
                "trim".to_owned(),
                "The connected file is a still image, with nothing that plays.".to_owned()
            ),
        ]
    );
}

#[test]
fn warns_about_a_take_with_no_sound_and_about_each_wrong_grid_picture_once() {
    let mut g = Graph::default();
    g.recorded_take(&[]);
    g.node("sound", "media.extractAudio", json!({}));
    g.node(
        "voice",
        "media.import",
        json!({ "path": "assets/voice.mp3" }),
    );
    g.node("grid", "image.grid", json!({}));
    g.link_ports("record", "sound", "recording", "media");
    g.link_ports("voice", "grid", "media", "in1");
    g.link_ports("voice", "grid", "media", "in2");
    g.link_ports("record", "grid", "recording", "in3");
    let results: IndexMap<String, NodeResult> = IndexMap::from([(
        "record".to_owned(),
        record_result(&make_log(Vec::new(), 2000.0)),
    )]);
    let mut found = warnings(&g.build(), Some(&results));
    found.sort();
    assert_eq!(
        found,
        [
            (
                "grid".to_owned(),
                "The connected file isn't an image.".to_owned()
            ),
            (
                "sound".to_owned(),
                "The connected file has no sound.".to_owned()
            ),
        ]
    );
}

#[test]
fn counts_a_converted_picture_as_a_file_on_the_cards() {
    let mut g = Graph::default();
    g.node("shot", "media.import", json!({ "path": "assets/shot.png" }));
    g.node("frame", "media.thumbnail", json!({}));
    g.node("music", "audio.music", json!({}));
    g.link_ports("frame", "music", "image", "media");
    g.link_ports("shot", "frame", "media", "media");
    let wf = g.build();
    let results: IndexMap<String, NodeResult> = IndexMap::from([(
        "frame".to_owned(),
        node_result(
            json!({ "outputs": { "image": { "type": "image", "src": "project:.opsis/cache/h/frame.png", "width": 640, "height": 360 } } }),
        ),
    )]);
    let compiled = compile_workflow(&wf, Some(&results));
    let cards = workflow_cards(
        &wf,
        &CardSources {
            compiled: &compiled,
            results: Some(&results),
            secrets: None,
            gone: &|_| false,
        },
    );
    assert_eq!(cards["music"][0].text, "The connected file has no sound");
}

#[test]
fn warns_before_resizing_or_animating_a_sound_and_before_speeding_up_a_still() {
    let mut g = Graph::default();
    g.node(
        "voice",
        "media.import",
        json!({ "path": "assets/voice.mp3" }),
    );
    g.node("shot", "media.import", json!({ "path": "assets/shot.png" }));
    for (id, kind, from) in [
        ("scaled", "media.scalePad", "voice"),
        ("gif", "media.gif", "voice"),
        ("fast", "media.speed", "shot"),
        ("back", "media.reverse", "shot"),
        ("faster-voice", "media.speed", "voice"),
        ("web", "media.convert", "voice"),
    ] {
        g.node(id, kind, json!({}));
        g.link_ports(from, id, "media", "media");
    }
    let mut found = warnings(&g.build(), None);
    found.sort();
    let picture = "The connected file is only sound, with no picture.";
    let plays = "The connected file is a still image, with nothing that plays.";
    assert_eq!(
        found,
        [
            ("back".to_owned(), plays.to_owned()),
            ("fast".to_owned(), plays.to_owned()),
            ("gif".to_owned(), picture.to_owned()),
            ("scaled".to_owned(), picture.to_owned()),
        ]
    );
}
