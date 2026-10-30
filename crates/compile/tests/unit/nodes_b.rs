use opsis_compile::clip::recording_props;
use opsis_compile::node::CardInputs;
use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::media::{QUALITY_PRESETS, loudness_name, loudness_presets, quality_name};
use opsis_compile::nodes::output::{Variant, variant_canvas, variant_size, video_file_names};
use opsis_compile::params::ParamSchema;
use opsis_compile::registry::node_def;
use opsis_types::workflow::WorkflowSettings;
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::helpers::{
    CLIP, card_texts, clip_value, compile_clip, node_card, node_result, parse_params, typed, zoomed,
};

fn size(variant: &str) -> (u32, u32) {
    let settings = WorkflowSettings {
        width: 1920,
        height: 1080,
        ..WorkflowSettings::default()
    };
    let s = variant_size(typed(json!(variant)), &settings);
    (s.width, s.height)
}

fn canvas(variant: &str) -> (u32, u32) {
    let s = variant_canvas(typed(json!(variant)), 1080);
    (s.width, s.height)
}

#[test]
fn sizes_export_shapes_by_their_long_or_short_edge() {
    assert_eq!(
        [size("16:9"), size("9:16"), size("1:1"), size("4:5")],
        [(1920, 1080), (1080, 1920), (1920, 1920), (1536, 1920)]
    );
    assert_eq!(
        [canvas("16:9"), canvas("1:1"), canvas("4:5"), canvas("9:16")],
        [(1920, 1080), (1080, 1080), (1080, 1350), (1080, 1920)]
    );
}

#[test]
fn shows_export_lengths_sizes_shapes_and_folders_on_cards() {
    let result = node_result(json!({
        "outputs": { "media": { "type": "media", "src": null, "kind": "video", "durationMs": 15_400 } },
        "artifacts": [{ "kind": "video", "path": "a.mp4", "bytes": 1_000_000 }, { "kind": "video", "path": "b.mp4", "bytes": 500_000 }]
    }));
    let def = node_def("output.render").unwrap();
    let texts = |variants: Value| -> Vec<String> {
        let params =
            crate::helpers::defaults_with("output.render", json!({ "variants": variants }));
        def.card_for(
            params.as_object().unwrap(),
            &Default::default(),
            &WorkflowSettings::default(),
            Some(&result),
            &CardInputs::default(),
        )
        .into_iter()
        .map(|r| r.text)
        .collect()
    };
    assert_eq!(
        texts(json!(["16:9", "9:16"])),
        ["16:9 + 9:16", "MP4 · 1080p · 30 fps", "15.4 s", "1.4 MiB"]
    );
    assert_eq!(
        texts(json!(["16:9", "1:1"]))[1],
        "MP4 · 1080p / 1920p · 30 fps"
    );
}

#[test]
fn names_export_files_stably_and_refuses_repeated_shapes() {
    let variants = |v: Value| -> Vec<Variant> { typed(v) };
    assert_eq!(
        video_file_names("demo", Codec::H264, &variants(json!(["16:9"]))),
        ["demo.mp4"]
    );
    assert_eq!(
        video_file_names("demo", Codec::H264, &variants(json!(["16:9", "9:16"]))),
        ["demo.mp4", "demo-9x16.mp4"]
    );
    assert!(parse_params("output.render", json!({ "variants": ["16:9", "16:9"] })).is_err());
    assert!(parse_params("output.still", json!({ "presets": ["og", "og"] })).is_err());
    let frames = CardInputs {
        frames: [("clip".to_owned(), 60)].into_iter().collect(),
        ..CardInputs::default()
    };
    assert!(
        card_texts("output.still", json!({ "atMs": 5000 }), frames.clone())
            .contains(&"The film ends at 2 s, so it uses the last frame".to_owned())
    );
    assert_eq!(
        card_texts("output.still", json!({ "atMs": 1000 }), frames).len(),
        2
    );
}

fn props_after(kind: &str, params: Value) -> Value {
    let clip = zoomed(Some(&[(kind, params)]), &[], None, json!({ "speed": 1 }));
    serde_json::to_value(recording_props(&clip)[0]).unwrap()
}

#[test]
fn uses_default_cursor_colours_when_cleared_and_none_for_the_mac_cursor() {
    let defaults = props_after("cursor.style", json!({}));
    assert_eq!(
        props_after("cursor.style", json!({ "color": "", "outline": " " }))["cursorLook"],
        defaults["cursorLook"]
    );
    let keys =
        props_after("cursor.keystrokeChips", json!({ "keycap": "", "ink": "" }))["keystrokes"]
            .clone();
    assert_eq!(
        (keys["keycap"].clone(), keys["ink"].clone()),
        (json!("#0a0a0a"), json!("#ffffff"))
    );
    assert_eq!(
        props_after("cursor.spotlight", json!({ "color": "" }))["annotations"][0]["color"],
        "rgba(0,0,0,0.55)"
    );
    assert_eq!(
        props_after("cursor.style", json!({ "look": "mac", "color": "#ff0000" }))["cursorLook"],
        json!({ "shape": "mac" })
    );
    let mac = node_card(
        "cursor.style",
        json!({ "look": "mac", "size": 1.5 }),
        CardInputs::default(),
    )[0]
    .clone();
    assert_eq!(
        (mac.text.as_str(), mac.swatches.is_none()),
        ("Mac · 1.5×", true)
    );
    assert_eq!(
        parse_params("cursor.style", json!({ "color": "not a colour" })).unwrap_err(),
        "Pick a colour"
    );
}

#[test]
fn asks_for_mark_text_only_where_marks_draw_text() {
    let problem = |kind: &str, text: &str| parse_params(kind, json!({ "text": text })).err();
    assert_eq!(
        problem("text.tooltip", " ").as_deref(),
        Some("Add the text to show")
    );
    assert_eq!(
        problem("text.calloutArrow", "").as_deref(),
        Some("Add the text to show")
    );
    assert_eq!(problem("cursor.spotlight", ""), None);
    assert!(
        node_def("cursor.highlightBox")
            .unwrap()
            .params
            .field("text")
            .is_none()
    );
    assert_eq!(
        card_texts(
            "cursor.spotlight",
            json!({ "match": "" }),
            CardInputs::default()
        )[0],
        "On every action"
    );
}

fn styled(style: &[(&str, Value)], own: &[(&str, Value)]) -> Value {
    serde_json::to_value(recording_props(&zoomed(Some(style), own, None, json!({ "speed": 1 })))[0])
        .unwrap()
}

#[test]
fn keeps_marks_in_chain_order_and_applies_style_steps_in_any_order() {
    let props = styled(
        &[
            ("cursor.spotlight", json!({})),
            ("cursor.highlightBox", json!({})),
            ("text.tooltip", json!({ "text": "Save" })),
        ],
        &[],
    );
    let kinds: Vec<Value> = props["annotations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["kind"].clone())
        .collect();
    assert_eq!(
        kinds,
        [json!("spotlight"), json!("highlight"), json!("tooltip")]
    );
    let style = [
        ("cursor.style", json!({ "size": 2 })),
        (
            "camera.backdrop",
            json!({ "kind": "solid", "color": "#ff0000" }),
        ),
    ];
    let zoom = [("camera.manualZoom", json!({ "atMs": 500 }))];
    let pick = |p: Value| {
        (
            p["cursorScale"].clone(),
            p["backdrop"].clone(),
            p["extraBeats"].as_array().unwrap().len(),
        )
    };
    let forward = pick(styled(&style, &zoom));
    assert_eq!(forward, (json!(2.0), json!("#ff0000"), 2));
    let reversed: Vec<(&str, Value)> = style.iter().rev().cloned().collect();
    assert_eq!(pick(styled(&reversed, &zoom)), forward);
}

#[test]
fn keeps_zooms_and_marks_made_before_a_laptop_frame_on_its_screen() {
    let clip = zoomed(
        Some(&[("cursor.spotlight", json!({}))]),
        &[("camera.manualZoom", json!({ "atMs": 500 }))],
        None,
        json!({ "speed": 1 }),
    );
    let framed = compile_clip(
        "camera.deviceFrame",
        json!({ "device": "laptop" }),
        json!({ CLIP: clip_value(&clip) }),
    );
    let props = serde_json::to_value(recording_props(&framed)[0]).unwrap();
    assert_eq!(props["screen"], true);
    assert_eq!(
        (
            props["extraBeats"].as_array().unwrap().len(),
            props["annotations"].as_array().unwrap().len()
        ),
        (2, 1)
    );
}

fn problem(schema: &ParamSchema, values: Value) -> Option<String> {
    schema
        .parse(values.as_object().unwrap(), None)
        .err()
        .map(|e| e.to_string())
}

#[test]
fn words_the_first_wrong_setting_in_the_unit_the_field_shows() {
    let schema = ParamSchema::new(json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Name" },
            "count": { "type": "number", "minimum": 1, "maximum": 3, "description": "Count" }
        },
        "required": ["name", "count"]
    }));
    let valid = json!({ "name": "a", "count": 2 });
    let with = |patch: Value| {
        let mut v = valid.clone();
        for (k, x) in patch.as_object().unwrap() {
            if x.is_null() {
                v.as_object_mut().unwrap().remove(k);
            } else {
                v[k] = x.clone();
            }
        }
        v
    };
    assert_eq!(problem(&schema, valid.clone()), None);
    assert_eq!(
        problem(&schema, with(json!({ "name": null }))).as_deref(),
        Some("Name is missing")
    );
    assert_eq!(
        problem(&schema, with(json!({ "name": 3 }))).as_deref(),
        Some("Name isn't valid")
    );
    assert_eq!(
        problem(&schema, with(json!({ "count": 0 }))).as_deref(),
        Some("Count must be at least 1")
    );
    assert_eq!(
        problem(&schema, with(json!({ "count": 9 }))).as_deref(),
        Some("Count must be at most 3")
    );
    assert_eq!(
        problem(&schema, with(json!({ "count": "{{Count}}" }))).as_deref(),
        Some("Count takes a number. Variables work only in text settings")
    );
    assert_eq!(
        problem(&schema, with(json!({ "count": "2" }))).as_deref(),
        Some("Count isn't valid")
    );
    let limits = ParamSchema::new(json!({
        "type": "object",
        "properties": {
            "waitMs": { "type": "number", "minimum": 500, "description": "Wait", "x-unit": "ms" },
            "lengthInFrames": { "type": "number", "minimum": 1, "description": "Length", "x-unit": "frames" },
            "zoom": { "type": "number", "maximum": 4, "description": "Zoom", "x-unit": "×" },
            "gap": { "type": "number", "exclusiveMinimum": 0, "description": "Gap", "x-unit": "px" },
            "values": { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 3, "description": "Values" },
            "fps": { "enum": [25, 30, 60], "description": "Frame rate", "x-unit": "fps" }
        }
    }));
    let ok = json!({ "waitMs": 500, "lengthInFrames": 1, "zoom": 1, "gap": 1, "values": [1, 2], "fps": 30 });
    let wrong = |patch: Value| {
        let mut v = ok.clone();
        v.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        problem(&limits, v)
    };
    assert_eq!(
        wrong(json!({ "waitMs": 100 })).as_deref(),
        Some("Wait must be at least 0.5 s")
    );
    assert_eq!(
        wrong(json!({ "lengthInFrames": 0 })).as_deref(),
        Some("Length is too short")
    );
    assert_eq!(
        wrong(json!({ "zoom": 5 })).as_deref(),
        Some("Zoom must be at most 4×")
    );
    assert_eq!(
        wrong(json!({ "gap": 0 })).as_deref(),
        Some("Gap must be more than 0 px")
    );
    assert_eq!(
        wrong(json!({ "values": [1] })).as_deref(),
        Some("Values needs at least 2")
    );
    assert_eq!(
        wrong(json!({ "values": [1, 2, 3, 4] })).as_deref(),
        Some("Values takes at most 3")
    );
    assert_eq!(
        wrong(json!({ "fps": 24 })).as_deref(),
        Some("Frame rate must be 25, 30 or 60 fps")
    );
}

#[test]
fn names_a_loudness_by_its_preset_or_in_lufs() {
    for (label, lufs) in loudness_presets() {
        assert_eq!(loudness_name(lufs), label);
    }
    assert_eq!(loudness_name(-18.0), "−18 LUFS");
}

proptest! {
    #[test]
    fn names_a_quality_after_the_nearest_preset_and_the_better_one_on_a_tie(crf in 0u32..=63) {
        let name = quality_name(crf);
        let (_, chosen) = QUALITY_PRESETS.iter().find(|(n, _)| *n == name).unwrap();
        for (_, other) in QUALITY_PRESETS {
            let (mine, theirs) = (chosen.abs_diff(crf), other.abs_diff(crf));
            prop_assert!(mine < theirs || (mine == theirs && *chosen <= other));
        }
    }

    #[test]
    fn writes_a_loudness_that_isnt_a_preset_in_lufs(lufs in -40i32..=-5) {
        let value = f64::from(lufs);
        prop_assume!(loudness_presets().iter().all(|(_, v)| *v != value));
        prop_assert_eq!(loudness_name(value), format!("−{} LUFS", -lufs));
    }
}
