use indexmap::IndexMap;
use opsis_compile::audio::sound_length;
use opsis_compile::clip::{ItemExtra, empty_clip, item_clip, recording_props, walk_items};
use opsis_compile::elements::elements;
use opsis_compile::elements::introspect::FieldWidget;
use opsis_compile::elements::overlay::captions::CAPTIONS_ELEMENT;
use opsis_compile::node::{CardInputs, InputMedia, PortSpec};
use opsis_compile::nodes::text::{ScriptPace, ends_sentence, script_to_captions};
use opsis_compile::nodes::transitions::presentations;
use opsis_compile::ports::{can_connect, convert_input};
use opsis_compile::registry::registry;
use opsis_compile::sounds::{BuiltinSong, BuiltinSound, Mood, builtin_songs, builtin_sounds};
use opsis_golden::typed;
use opsis_types::cards::Tone;
use opsis_types::numbers::whole;
use opsis_types::timeline::TimelineItem;
use opsis_types::values::{CompileValue, EntriesValue, MediaKind, StyleEntry};
use opsis_types::workflow::PortType;
use serde_json::{Map, Value, json};

use crate::helpers::{
    CLIP, card_texts, clip_value, compile_clip, compile_node, compile_node_with, node_card,
    node_card_with, node_result, parse_params, zoomed,
};

#[test]
fn starts_every_node_with_parseable_defaults_and_a_hidden_role_beside_each_picked_element() {
    let mut unparsed = Vec::new();
    let mut missing_roles = Vec::new();
    let mut unknown_widgets = Vec::new();
    for def in registry().all() {
        let defaults = def.defaults();
        let required: Vec<&str> = def.params.json()["required"]
            .as_array()
            .map(|r| r.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        for key in def.params.keys() {
            let field = def.params.field(key).unwrap();
            let widget = field["x-widget"].as_str();
            if !defaults.contains_key(key)
                && required.contains(&key.as_str())
                && widget != Some("target")
            {
                unparsed.push(format!("{}.{key}", def.kind));
            }
            if widget == Some("target")
                && def
                    .params
                    .field(&format!("{key}Role"))
                    .is_none_or(|r| r["x-hidden"] != true)
            {
                missing_roles.push(format!("{}.{key}", def.kind));
            }
            if let Some(w) = widget.filter(|w| FieldWidget::named(w).is_none()) {
                unknown_widgets.push(format!("{}.{key}: {w}", def.kind));
            }
        }
    }
    assert!(unparsed.is_empty(), "{unparsed:?}");
    assert!(missing_roles.is_empty(), "{missing_roles:?}");
    assert!(unknown_widgets.is_empty(), "{unknown_widgets:?}");
}

#[test]
fn generates_one_node_per_placeable_registry_element() {
    let mut placed: Vec<String> = registry()
        .all()
        .iter()
        .filter_map(|d| d.element.clone())
        .collect();
    let mut expected: Vec<String> = elements()
        .iter()
        .filter(|e| e.key != CAPTIONS_ELEMENT)
        .map(|e| e.key.clone())
        .collect();
    placed.sort();
    expected.sort();
    assert_eq!(placed, expected);
}

fn recording() -> CompileValue {
    typed(
        &json!({ "type": "recording", "src": "project:take.mp4", "log": null, "durationMs": 4000, "viewport": { "width": 1280, "height": 720 }, "nodeId": "rec", "style": [] }),
    )
}

fn spec(kind: PortType, label: Option<&'static str>) -> PortSpec {
    PortSpec {
        kind,
        label: label.map(Into::into),
        optional: false,
        many: false,
        need: None,
    }
}

#[test]
fn connects_the_same_resource_and_converts_images_and_recordings_to_files() {
    assert!(
        can_connect(PortType::BrowserSession, PortType::BrowserSession)
            && can_connect(PortType::RecordingStyle, PortType::RecordingStyle)
    );
    assert!(
        can_connect(PortType::Image, PortType::Media)
            && can_connect(PortType::Recording, PortType::Media)
    );
    for (from, to) in [
        (PortType::Media, PortType::Audio),
        (PortType::Recording, PortType::Clip),
    ] {
        assert!(!can_connect(from, to), "{from:?} -> {to:?}");
    }
    let converted = serde_json::to_value(
        convert_input(Some(&recording()), Some(&spec(PortType::Media, None))).unwrap(),
    )
    .unwrap();
    assert_eq!(
        converted,
        json!({ "type": "media", "kind": "video", "src": "project:take.mp4", "durationMs": 4000, "width": 1280, "height": 720, "hasAudio": false })
    );
    let image: CompileValue =
        typed(&json!({ "type": "image", "src": "project:s.png", "width": 3, "height": 2 }));
    let converted = serde_json::to_value(
        convert_input(Some(&image), Some(&spec(PortType::Media, None))).unwrap(),
    )
    .unwrap();
    assert_eq!(
        converted,
        json!({ "type": "media", "kind": "image", "src": "project:s.png", "width": 3, "height": 2 })
    );
    assert_eq!(
        convert_input(Some(&recording()), Some(&spec(PortType::Recording, None))),
        Some(recording())
    );
    assert_eq!(
        convert_input(Some(&recording()), Some(&spec(PortType::Clip, None))),
        None
    );
    assert_eq!(
        [
            spec(PortType::Audio, None).label(),
            spec(PortType::RecordingStyle, None).label(),
            spec(PortType::Clip, Some("Intro")).label()
        ],
        ["Sound", "Style", "Intro"]
    );
}

fn frames(pairs: &[(&str, u32)]) -> CardInputs {
    CardInputs {
        frames: pairs.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
        ..CardInputs::default()
    }
}

fn leak_in(a: u32, b: u32, length: u32) -> (u32, u32, u32) {
    let film = compile_clip(
        "transition.lightLeak",
        json!({ "durationInFrames": length }),
        json!({ "a": clip_value(&empty_clip(a)), "b": clip_value(&empty_clip(b)) }),
    );
    let leak = film.tracks[1].items[0].base();
    (film.duration_in_frames, leak.from, leak.duration_in_frames)
}

#[test]
fn centres_a_light_leak_on_the_cut_and_keeps_a_long_one_inside_the_film() {
    assert_eq!(leak_in(90, 90, 30), (180, 75, 30));
    assert_eq!(leak_in(30, 30, 90), (60, 0, 60));
    assert_eq!(leak_in(10, 90, 40), (100, 0, 40));
    assert_eq!(
        card_texts(
            "transition.lightLeak",
            json!({ "durationInFrames": 90 }),
            frames(&[("a", 30), ("b", 30)])
        ),
        ["3 s", "Longer than both clips, so it plays for 2 s"]
    );
    assert_eq!(
        card_texts(
            "transition.lightLeak",
            json!({ "durationInFrames": 90 }),
            frames(&[("a", 60), ("b", 60)])
        ),
        ["3 s"]
    );
}

#[test]
fn every_transition_passes_a_lone_clip_through_unchanged() {
    let kinds: Vec<String> = presentations()
        .map(|m| format!("transition.{}", m.key))
        .collect();
    assert!(kinds.len() > 10);
    let (first, then) = (empty_clip(40), empty_clip(70));
    for kind in kinds {
        assert_eq!(
            compile_clip(&kind, json!({}), json!({ "a": clip_value(&first) })),
            first,
            "{kind}"
        );
        assert_eq!(
            compile_clip(&kind, json!({}), json!({ "b": clip_value(&then) })),
            then,
            "{kind}"
        );
    }
}

fn sfx_item(params: Value) -> opsis_types::timeline::AudioItem {
    match compile_node("audio.sfx", params, json!({})).shift_remove("audio") {
        Some(CompileValue::Audio(audio)) => audio.items[0].clone(),
        other => panic!("no sound: {other:?}"),
    }
}

#[test]
fn knows_how_long_a_sound_effect_plays() {
    assert_eq!(
        sfx_item(json!({ "src": "assets/tone.wav", "srcMs": 3000 })).length_frames,
        Some(90)
    );
    let unknown = sfx_item(json!({ "src": "assets/tone.wav" }));
    assert_eq!(
        (unknown.length_frames, sound_length(&unknown, 150)),
        (None, 150.0)
    );
    assert!(
        sfx_item(json!({ "src": builtin_sounds()[0].reference() }))
            .length_frames
            .unwrap()
            > 0
    );
}

fn made_sound_result(file: &str, duration_ms: u32) -> opsis_types::values::NodeResult {
    node_result(json!({ "outputs": { "audio": {
        "type": "media", "kind": "audio", "src": format!("project:.opsis/cache/h/{file}"),
        "durationMs": duration_ms, "hasAudio": true
    } } }))
}

fn made_item(
    kind: &str,
    params: Value,
    file: &str,
    duration_ms: u32,
) -> opsis_types::timeline::AudioItem {
    let result = made_sound_result(file, duration_ms);
    match compile_node_with(kind, params, json!({}), Some(&result))
        .unwrap()
        .shift_remove("audio")
    {
        Some(CompileValue::Audio(audio)) => audio.items[0].clone(),
        other => panic!("no sound: {other:?}"),
    }
}

#[test]
fn loops_room_tone_under_the_whole_film_with_its_fades() {
    let item = made_item(
        "audio.roomTone",
        json!({ "fadeInMs": 500, "fadeOutMs": 2000 }),
        "room-tone.wav",
        10_000,
    );
    assert_eq!(
        (
            item.from,
            item.looped,
            item.fade_in_frames,
            item.fade_out_frames
        ),
        (0, Some(true), Some(15), Some(60))
    );
    assert_eq!(item.role, Some(opsis_types::timeline::AudioRole::Music));
    assert_eq!(
        (item.channel.as_deref(), item.keep_music),
        (Some("n"), Some(true))
    );
    match compile_node("audio.roomTone", json!({}), json!({})).shift_remove("audio") {
        Some(CompileValue::Audio(audio)) => assert_eq!(audio.items, []),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        card_texts(
            "audio.roomTone",
            json!({ "colour": "brown", "level": -50 }),
            CardInputs::default()
        ),
        ["Brown noise · -50 dB", "Made when the node runs"]
    );
    let made = made_sound_result("room-tone.wav", 10_000);
    assert_eq!(
        node_card_with(
            "audio.roomTone",
            json!({}),
            CardInputs::default(),
            Some(&made)
        )
        .len(),
        1
    );
}

#[test]
fn places_tones_at_a_time_or_on_an_action_for_their_own_length() {
    let at = made_item("audio.tones", json!({ "atMs": 2000 }), "tones.wav", 150);
    assert_eq!((at.from, at.length_frames, at.anchor), (60, Some(5), None));
    assert_eq!(at.role, Some(opsis_types::timeline::AudioRole::Effect));
    let on = made_item(
        "audio.tones",
        json!({ "kind": "countdown", "atAction": "click", "offsetMs": -3000 }),
        "tones.wav",
        3450,
    );
    assert_eq!(on.from, -90);
    assert_eq!(on.anchor.map(|a| a.node_id).as_deref(), Some("click"));
    assert_eq!(
        card_texts(
            "audio.tones",
            json!({ "kind": "countdown", "count": 3 }),
            CardInputs::default()
        ),
        ["Countdown · 3 beeps", "At 0 s", "Made when the node runs"]
    );
}

#[test]
fn lists_what_clean_speech_does_and_the_noise_before_and_after_a_run() {
    assert_eq!(
        card_texts("media.cleanSpeech", json!({}), CardInputs::default()),
        [
            "Removes noise, rumble and harsh “s”, evens the level",
            "For speech: music under the voice goes too"
        ]
    );
    assert_eq!(
        card_texts(
            "media.cleanSpeech",
            json!({ "noise": 0, "softenS": 0, "evenLevel": false }),
            CardInputs::default()
        ),
        ["Removes rumble"]
    );
    assert_eq!(
        card_texts(
            "media.cleanSpeech",
            json!({ "method": "spectral", "softenS": 0 }),
            CardInputs::default()
        ),
        ["Removes noise and rumble, evens the level"]
    );
    let made = node_result(json!({
        "outputs": { "media": { "type": "media", "kind": "audio", "src": "project:.opsis/cache/h/speech.wav", "durationMs": 4000 } },
        "measured": [{ "type": "noiseLevel", "beforeDb": -41.4, "afterDb": -68.6 }]
    }));
    let rows: Vec<String> = node_card_with(
        "media.cleanSpeech",
        json!({ "method": "spectral" }),
        CardInputs::default(),
        Some(&made),
    )
    .into_iter()
    .map(|r| r.text)
    .collect();
    assert_eq!(rows[1], "Noise -41 dB → -69 dB");
    assert_eq!(rows.len(), 3);
}

#[test]
fn says_how_before_and_after_lays_out_and_the_size_it_will_make() {
    let mut inputs = CardInputs::default();
    inputs.pixels.insert("image".to_owned(), (1200, 800));
    inputs.pixels.insert("after".to_owned(), (600, 800));
    inputs.connected.push("image".to_owned());
    inputs.connected.push("after".to_owned());
    assert_eq!(
        card_texts("image.beforeAfter", json!({}), inputs.clone()),
        ["Split at 50% · “Before” and “After”", "1200 × 800 px"]
    );
    assert_eq!(
        card_texts(
            "image.beforeAfter",
            json!({ "layout": "side-by-side", "labels": false, "dividerWidth": 10 }),
            inputs.clone()
        ),
        ["Side by side", "1810 × 800 px"]
    );
    assert_eq!(
        card_texts(
            "image.beforeAfter",
            json!({ "angle": 12, "position": 0.3 }),
            CardInputs::default()
        )[..2],
        [
            "Split at 30%, turned 12° · “Before” and “After”",
            "Connect the after picture"
        ]
    );
}

fn magnify_with(params: Value) -> opsis_compile::nodes::image::Magnify {
    typed(&crate::helpers::defaults_with("image.magnify", params))
}

#[test]
fn sizes_the_magnified_view_from_whichever_two_settings_the_mode_names() {
    use opsis_compile::nodes::image::{Rect, magnify_plan};
    let source = (1000.0, 500.0);
    let boxes = json!({
        "area": [{ "x": 0.1, "y": 0.1, "width": 0.1, "height": 0.2 }],
        "shown": [{ "x": 0.5, "y": 0.2, "width": 0.3, "height": 0.6 }],
        "zoom": 2.0
    });
    let two = magnify_plan(source, &magnify_with(boxes.clone()));
    assert_eq!(
        two.area,
        Rect {
            x: 100.0,
            y: 50.0,
            width: 100.0,
            height: 100.0
        }
    );
    assert_eq!(
        two.shown,
        Rect {
            x: 500.0,
            y: 100.0,
            width: 300.0,
            height: 300.0
        }
    );
    assert_eq!(two.zoom, 3.0);
    let mut by_area = boxes.clone();
    by_area["sizeFrom"] = json!("area-and-zoom");
    let area_zoom = magnify_plan(source, &magnify_with(by_area));
    assert_eq!(
        area_zoom.shown,
        Rect {
            x: 550.0,
            y: 150.0,
            width: 200.0,
            height: 200.0
        }
    );
    assert_eq!(area_zoom.zoom, 2.0);
    let mut by_shown = boxes;
    by_shown["sizeFrom"] = json!("shown-and-zoom");
    let shown_zoom = magnify_plan(source, &magnify_with(by_shown));
    assert_eq!(
        shown_zoom.area,
        Rect {
            x: 75.0,
            y: 25.0,
            width: 150.0,
            height: 150.0
        }
    );
    assert_eq!(shown_zoom.zoom, 2.0);
    let edge = magnify_plan(
        source,
        &magnify_with(json!({
            "sizeFrom": "area-and-zoom",
            "area": [{ "x": 0.0, "y": 0.0, "width": 0.1, "height": 0.2 }],
            "shown": [{ "x": 0.9, "y": 0.9, "width": 0.1, "height": 0.1 }],
            "zoom": 4.0
        })),
    );
    assert_eq!(
        edge.shown,
        Rect {
            x: 600.0,
            y: 100.0,
            width: 400.0,
            height: 400.0
        },
        "kept inside the picture"
    );
    let rounded = magnify_plan(
        source,
        &magnify_with(
            json!({ "shape": "rounded", "area": [{ "x": 0.1, "y": 0.1, "width": 0.1, "height": 0.1 }], "shown": [{ "x": 0.5, "y": 0.2, "width": 0.4, "height": 0.2 }] }),
        ),
    );
    assert_eq!(
        (rounded.area.width, rounded.area.height, rounded.zoom),
        (200.0, 50.0, 2.0),
        "the area takes the shape of the view"
    );
}

#[test]
fn says_the_zoom_and_shape_on_the_magnify_card() {
    let mut inputs = CardInputs::default();
    inputs.pixels.insert("image".to_owned(), (1000, 500));
    assert_eq!(
        card_texts(
            "image.magnify",
            json!({ "area": [{ "x": 0.1, "y": 0.1, "width": 0.1, "height": 0.2 }], "shown": [{ "x": 0.5, "y": 0.2, "width": 0.3, "height": 0.6 }] }),
            inputs
        ),
        ["3× · circle", "1000 × 500 px"]
    );
    assert_eq!(
        card_texts(
            "image.magnify",
            json!({ "sizeFrom": "shown-and-zoom", "shape": "rounded" }),
            CardInputs::default()
        ),
        ["2.5× · rounded box"]
    );
    assert_eq!(
        card_texts("image.magnify", json!({}), CardInputs::default()),
        ["Two boxes · circle"]
    );
}

#[test]
fn counts_the_changes_on_the_compare_card_and_warns_about_different_sizes() {
    let found = |count: u32| {
        node_result(json!({ "outputs": {}, "measured": [{ "type": "changes", "count": count }] }))
    };
    let mut inputs = CardInputs::default();
    inputs.connected.push("after".to_owned());
    let texts =
        |result: Option<&opsis_types::values::NodeResult>, inputs: CardInputs| -> Vec<String> {
            node_card_with("image.compare", json!({}), inputs, result)
                .into_iter()
                .map(|r| r.text)
                .collect()
        };
    assert_eq!(texts(Some(&found(5)), inputs.clone()), ["5 changes"]);
    assert_eq!(texts(Some(&found(0)), inputs.clone()), ["No change"]);
    assert_eq!(texts(None, inputs.clone()), ["Marks what changed"]);
    inputs.pixels.insert("image".to_owned(), (1280, 720));
    inputs.pixels.insert("after".to_owned(), (1280, 800));
    assert_eq!(
        texts(None, inputs),
        ["Marks what changed", "The two pictures are different sizes"]
    );
    assert_eq!(
        texts(None, CardInputs::default()),
        ["Marks what changed", "Connect the after picture"]
    );
}

#[test]
fn says_how_many_semitones_the_pitch_moves() {
    let texts = |params: Value| card_texts("media.pitch", params, CardInputs::default());
    assert_eq!(texts(json!({})), ["+3 semitones"]);
    assert_eq!(
        texts(json!({ "semitones": -1, "keepCharacter": false })),
        ["−1 semitone", "The voice's character changes too"]
    );
    assert_eq!(texts(json!({ "semitones": 0 })), ["No change"]);
}

#[test]
fn names_the_voice_effect_and_its_mix() {
    let texts = |params: Value| card_texts("media.voiceEffect", params, CardInputs::default());
    assert_eq!(texts(json!({})), ["Telephone · 100% mixed"]);
    assert_eq!(
        texts(json!({ "effect": "hall", "mix": 0.3 })),
        ["Hall · 30% mixed"]
    );
    assert_eq!(
        texts(json!({ "effect": "echo", "delayMs": 250, "mix": 0.4 })),
        ["Echo after 0.25 s · 40% mixed"]
    );
}

#[test]
fn stretches_a_song_within_the_limit_and_loops_or_cuts_it_otherwise() {
    use opsis_compile::nodes::media::{FitPlan, fit_plan};
    let fit = |params: Value| {
        typed::<opsis_compile::nodes::media::FitMusic>(&crate::helpers::defaults_with(
            "media.fitMusic",
            params,
        ))
    };
    assert_eq!(
        fit_plan(100_000.0, 105_000.0, &fit(json!({}))),
        FitPlan::Stretch {
            tempo: 100.0 / 105.0
        }
    );
    assert_eq!(
        fit_plan(
            100_000.0,
            250_000.0,
            &fit(json!({ "loopFromMs": 20_000, "loopToMs": 80_000 }))
        ),
        FitPlan::Loop {
            from_ms: 20_000.0,
            to_ms: 80_000.0,
            repeats: 3
        }
    );
    assert_eq!(fit_plan(100_000.0, 60_000.0, &fit(json!({}))), FitPlan::Cut);
    assert_eq!(
        fit_plan(100_000.0, 60_000.0, &fit(json!({ "way": "stretch" }))),
        FitPlan::Stretch {
            tempo: 100.0 / 60.0
        }
    );
    assert_eq!(
        fit_plan(100_000.0, 103_000.0, &fit(json!({ "way": "loop" }))),
        FitPlan::Loop {
            from_ms: 0.0,
            to_ms: 100_000.0,
            repeats: 1
        }
    );
    assert_eq!(
        fit_plan(10_000.0, 30_000.0, &fit(json!({ "loopFromMs": 9_800 }))),
        FitPlan::Loop {
            from_ms: 0.0,
            to_ms: 10_000.0,
            repeats: 3
        },
        "a loop too short for its crossfade takes the whole song"
    );
}

#[test]
fn says_how_the_song_was_fitted() {
    let after = |measured: Value| {
        let result = node_result(json!({ "outputs": {}, "measured": [measured] }));
        let mut inputs = CardInputs::default();
        inputs.connected.push("clip".to_owned());
        inputs.connected.push("media".to_owned());
        node_card_with("media.fitMusic", json!({}), inputs, Some(&result))
            .into_iter()
            .map(|r| r.text)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        after(json!({ "type": "fitted", "tempo": 0.96, "loops": 0 })),
        ["From the connected file", "Stretched 4%"]
    );
    assert_eq!(
        after(json!({ "type": "fitted", "tempo": 1.05, "loops": 0 }))[1],
        "Sped up 5%"
    );
    assert_eq!(
        after(json!({ "type": "fitted", "loops": 2 }))[1],
        "Looped twice"
    );
    assert_eq!(
        after(json!({ "type": "fitted", "loops": 0 }))[1],
        "Cut and faded"
    );
    assert_eq!(
        card_texts("media.fitMusic", json!({}), CardInputs::default()),
        ["Choose a song", "Connect the film for its length"]
    );
}

#[test]
fn counts_the_marks_and_steps_and_warns_about_empty_text() {
    let texts = |marks: Value| {
        card_texts(
            "image.annotate",
            json!({ "marks": marks }),
            CardInputs::default(),
        )
    };
    assert_eq!(texts(json!([])), ["Draw marks on the picture"]);
    assert_eq!(
        texts(json!([{ "kind": "step" }, { "kind": "arrow" }, { "kind": "step" }])),
        ["3 marks · 2 steps"]
    );
    assert_eq!(
        texts(json!([{ "kind": "text", "text": " " }])),
        ["1 mark", "A text mark has no text"]
    );
}

#[test]
fn starts_each_generated_sound_at_the_pick_and_reaches_the_other_variations() {
    use opsis_compile::nodes::audio::{GenerateSound, SoundPreset, sound_shape};
    let shape = |params: Value| {
        sound_shape(&typed::<GenerateSound>(&crate::helpers::defaults_with(
            "audio.generate",
            params,
        )))
    };
    let picks = [
        ("whoosh", 0.8, 0.0, 0.15),
        ("riser", 3.0, -5.0, 0.5),
        ("pop", 0.15, 5.0, 0.5),
        ("ding", 1.5, 5.0, 0.5),
        ("tap", 0.05, 5.0, 0.5),
        ("swell", 3.0, 0.0, 0.5),
        ("click", 0.1, 0.0, 0.9),
    ];
    for (preset, seconds, pitch, brightness) in picks {
        let s = shape(json!({ "preset": preset }));
        assert_eq!(
            (s.seconds, s.pitch, s.brightness, s.seed),
            (seconds, pitch, brightness, 1),
            "{preset}"
        );
    }
    assert_eq!(SoundPreset::ALL.len(), picks.len());
    let riser = |variation: &str| shape(json!({ "preset": "riser", "variation": variation }));
    assert_eq!(
        (
            riser("as-made").pitch,
            riser("darker").brightness,
            riser("darker").pitch
        ),
        (0.0, 0.15, 0.0)
    );
    assert_eq!(
        (
            riser("another-take").seed,
            riser("higher").pitch,
            riser("shorter").seconds,
            riser("longer").seconds
        ),
        (2, 5.0, 1.5, 6.0)
    );
    let click = |variation: &str| shape(json!({ "preset": "click", "variation": variation }));
    assert_eq!(
        (
            click("shorter").seed,
            click("longer").seed,
            click("longer").seconds
        ),
        (0, 4, 0.1)
    );
    let custom = shape(
        json!({ "preset": "pop", "variation": "custom", "lengthMs": 400, "pitch": -2.5, "brightness": 0.7, "take": 9 }),
    );
    assert_eq!(
        (custom.seconds, custom.pitch, custom.brightness, custom.seed),
        (0.4, -2.5, 0.7, 9)
    );
    assert_eq!(
        shape(json!({ "preset": "pop", "variation": "custom" })).seconds,
        0.15
    );
}

#[test]
fn places_a_generated_sound_like_a_sound_effect_and_names_it_on_the_card() {
    let at = made_item(
        "audio.generate",
        json!({ "atMs": 1000 }),
        "generated.wav",
        800,
    );
    assert_eq!((at.from, at.length_frames), (30, Some(24)));
    let on = made_item(
        "audio.generate",
        json!({ "atAction": "go", "offsetMs": -500 }),
        "generated.wav",
        800,
    );
    assert_eq!(
        (on.from, on.anchor.map(|a| a.node_id)),
        (-15, Some("go".to_owned()))
    );
    assert_eq!(
        card_texts(
            "audio.generate",
            json!({ "preset": "riser", "variation": "lower" }),
            CardInputs::default()
        ),
        [
            "Riser · 5 semitones lower · 3 s",
            "At 0 s",
            "Made when the node runs"
        ]
    );
}

#[test]
fn plays_a_built_in_song_for_its_listed_length_and_names_it_on_the_card() {
    let song = &builtin_songs()[0];
    match compile_node("audio.music", json!({ "src": song.reference() }), json!({}))
        .shift_remove("audio")
    {
        Some(CompileValue::Audio(audio)) => {
            assert_eq!(audio.items[0].src, song.reference());
            assert_eq!(
                audio.duration_in_frames,
                Some(whole(f64::from(song.duration_ms) * 30.0 / 1000.0))
            );
        }
        other => panic!("{other:?}"),
    }
    let first = node_card(
        "audio.music",
        json!({ "src": song.reference() }),
        CardInputs::default(),
    )[0]
    .clone();
    assert!(first.text.contains(&song.label), "{}", first.text);
}

#[test]
fn ships_a_file_for_every_built_in_sound_and_song() {
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../public");
    let refs = builtin_sounds()
        .iter()
        .map(BuiltinSound::reference)
        .chain(builtin_songs().iter().map(BuiltinSong::reference));
    let missing: Vec<String> = refs
        .filter(|r| {
            let file = public.join(r.trim_start_matches("app:"));
            std::fs::metadata(file).map_or(true, |m| m.len() == 0)
        })
        .collect();
    assert_eq!(missing, Vec::<String>::new());
    assert_eq!(
        builtin_songs().iter().map(|s| s.mood).collect::<Vec<_>>(),
        [Mood::Upbeat, Mood::Calm, Mood::Ambient]
    );
}

#[test]
fn leaves_out_a_sound_that_starts_at_or_after_the_clips_end() {
    let sounds = |at_ms: f64| {
        let audio = compile_node(
            "audio.sfx",
            json!({ "src": builtin_sounds()[0].reference(), "atMs": at_ms }),
            json!({}),
        )
        .shift_remove("audio")
        .unwrap();
        compile_clip(
            "audio.mix",
            json!({}),
            json!({ CLIP: clip_value(&empty_clip(90)), "sounds": audio }),
        )
        .audio
        .len()
    };
    assert_eq!([sounds(1000.0), sounds(3000.0), sounds(5000.0)], [1, 0, 0]);
}

#[test]
fn adds_no_sound_from_a_file_that_has_none_and_says_so() {
    let silent =
        json!({ "type": "media", "src": "project:a.mp4", "kind": "video", "hasAudio": false });
    let image = json!({ "type": "media", "src": "project:a.png", "kind": "image" });
    for media in [&silent, &image] {
        for kind in ["audio.music", "audio.sfx"] {
            match compile_node(kind, json!({}), json!({ "media": media })).shift_remove("audio") {
                Some(CompileValue::Audio(audio)) => assert!(audio.items.is_empty(), "{kind}"),
                other => panic!("{kind}: {other:?}"),
            }
        }
    }
    let card = |kind: MediaKind, has_audio: Option<bool>, src: Value| {
        let inputs = CardInputs {
            connected: vec!["media".into()],
            media: IndexMap::from([("media".to_owned(), InputMedia { kind, has_audio })]),
            ..CardInputs::default()
        };
        node_card("audio.music", src, inputs)[0].clone()
    };
    assert_eq!(
        card(MediaKind::Video, Some(false), json!({})).text,
        "The connected file has no sound"
    );
    assert_eq!(
        card(MediaKind::Image, None, json!({})).text,
        "The connected file has no sound"
    );
    let inputs = CardInputs {
        connected: vec!["media".into()],
        media: IndexMap::from([(
            "media".to_owned(),
            InputMedia {
                kind: MediaKind::Audio,
                has_audio: Some(true),
            },
        )]),
        ..CardInputs::default()
    };
    let first = node_card(
        "audio.sfx",
        json!({ "src": builtin_sounds()[0].reference() }),
        inputs,
    )[0]
    .clone();
    assert_eq!(
        (first.text, first.tone),
        (
            format!(
                "Plays the connected file, not “{}”",
                builtin_sounds()[0].label
            ),
            Some(Tone::Warn)
        )
    );
}

fn redacted(params: &[Value]) -> Value {
    let own: Vec<(&str, Value)> = params
        .iter()
        .map(|p| ("effect.redactTake", p.clone()))
        .collect();
    serde_json::to_value(recording_props(&zoomed(None, &own, None, json!({ "speed": 1 })))[0])
        .unwrap()
}

#[test]
fn puts_redactions_on_the_takes_footage_in_order_across_nodes() {
    let props = redacted(&[
        json!({ "style": "solid", "fromMs": 1000, "untilMs": 2000 }),
        json!({ "style": "blur", "fromMs": 3000 }),
    ]);
    let entries = props["footage"]["redact"].as_array().unwrap();
    let shown: Vec<(Value, Value, Value)> = entries
        .iter()
        .map(|e| {
            (
                e["params"]["style"].clone(),
                e["params"]["fromMs"].clone(),
                e["params"]["untilMs"].clone(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (json!("solid"), json!(1000), json!(2000)),
            (json!("blur"), json!(3000), Value::Null)
        ]
    );
    assert!(entries.iter().all(|e| e["type"] == "effect.redactTake"));
    assert_eq!(
        parse_params(
            "effect.redactTake",
            json!({ "fromMs": 2000, "untilMs": 1000 })
        )
        .unwrap_err(),
        "Until must come after From"
    );
    assert_eq!(
        card_texts(
            "effect.redactTake",
            json!({ "fromMs": 500 }),
            CardInputs::default()
        ),
        ["1 area pixelated", "From 0.5 s"]
    );
}

fn element_props(clip: &opsis_types::values::ClipValue) -> Vec<Map<String, Value>> {
    let mut out = Vec::new();
    walk_items(&clip.tracks, &mut |i| {
        if let TimelineItem::Element { props, .. } = i {
            out.push(props.clone());
        }
    });
    out
}

fn grade_value(kind: &str) -> Value {
    serde_json::to_value(CompileValue::Grade(EntriesValue {
        entries: vec![StyleEntry {
            node_id: "g".into(),
            kind: kind.into(),
            params: Map::new(),
        }],
    }))
    .unwrap()
}

#[test]
fn grades_the_footage_of_recordings_videos_and_images_only() {
    let grade = grade_value("grade.colour");
    let take = zoomed(None, &[], None, json!({ "speed": 1 }));
    let graded = compile_clip(
        "effect.gradeFootage",
        json!({}),
        json!({ CLIP: clip_value(&take), "grade": grade }),
    );
    let footage = serde_json::to_value(recording_props(&graded)[0]).unwrap()["footage"].clone();
    assert_eq!(footage["grade"][0]["type"], "grade.colour");
    for (id, component, src) in [
        ("v", "video", "project:a.mp4"),
        ("i", "image", "project:a.png"),
    ] {
        let clip = item_clip(
            id,
            component,
            json!({ "src": src }).as_object().unwrap().clone(),
            30,
            ItemExtra::default(),
        );
        let once = compile_clip(
            "effect.gradeFootage",
            json!({}),
            json!({ CLIP: clip_value(&clip), "grade": grade.clone() }),
        );
        let twice = compile_clip(
            "effect.gradeFootage",
            json!({}),
            json!({ CLIP: clip_value(&once), "grade": grade_value("grade.mono") }),
        );
        let steps: Vec<Value> = element_props(&twice)[0]["footage"]["grade"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["type"].clone())
            .collect();
        assert_eq!(
            steps,
            [json!("grade.colour"), json!("grade.mono")],
            "{component}"
        );
    }
    let title = item_clip(
        "t",
        "title",
        json!({ "title": "Hi" }).as_object().unwrap().clone(),
        30,
        ItemExtra::default(),
    );
    assert!(
        !element_props(&compile_clip(
            "effect.gradeFootage",
            json!({}),
            json!({ CLIP: clip_value(&title), "grade": grade })
        ))[0]
            .contains_key("footage")
    );
    assert_eq!(
        card_texts("effect.gradeFootage", json!({}), CardInputs::default())[0],
        "Connect a grade"
    );
    let mut connected = CardInputs::default();
    connected.grade.insert(
        "grade".into(),
        vec![StyleEntry {
            node_id: "g".into(),
            kind: "grade.mono".into(),
            params: Map::new(),
        }],
    );
    assert_eq!(
        card_texts("effect.gradeFootage", json!({}), connected)[0],
        "1 step: Black and white and tint"
    );
}

fn pace(words_per_minute: f64, start_ms: f64, line_pause_ms: f64) -> ScriptPace {
    ScriptPace {
        words_per_minute,
        start_ms,
        line_pause_ms,
        paragraph_pause_ms: line_pause_ms * 2.0,
    }
}

#[test]
fn times_each_word_at_the_speaking_pace_and_pauses_after_sentences_and_paragraphs() {
    let timed: Vec<(String, f64, f64)> =
        script_to_captions("Record it. Then  share", pace(120.0, 1000.0, 300.0))
            .into_iter()
            .map(|c| (c.text, c.start_ms, c.end_ms))
            .collect();
    let expected = [
        ("Record", 1000.0, 1500.0),
        ("it.", 1500.0, 2000.0),
        ("Then", 2300.0, 2800.0),
        ("share", 2800.0, 3300.0),
    ]
    .map(|(t, a, b)| (t.to_owned(), a, b));
    assert_eq!(timed, expected);
    assert_eq!(script_to_captions("  ", pace(160.0, 0.0, 0.0)), []);
    assert!(
        ["it.\"", "(see it.)", "wait…", "Done!”", "end。"]
            .iter()
            .all(|w| ends_sentence(w))
    );
    assert!(
        !["e.g.", "i.e.", "Dr.", "etc."]
            .iter()
            .any(|w| ends_sentence(w))
    );
    let starts: Vec<(String, f64)> = script_to_captions("One\n\nTwo", pace(120.0, 0.0, 300.0))
        .into_iter()
        .map(|c| (c.text, c.start_ms))
        .collect();
    assert_eq!(
        starts,
        [("One".to_owned(), 0.0), ("Two".to_owned(), 1100.0)]
    );
    assert!(script_to_captions("今日はいい天気です。", pace(120.0, 0.0, 300.0)).len() > 1);
}

#[test]
fn shows_captions_until_read_or_for_the_clip_and_warns_when_they_run_past_it() {
    let captions =
        json!({ "type": "captions", "captions": [{ "text": "Hi", "startMs": 0, "endMs": 2500 }] });
    assert_eq!(
        compile_clip("text.captions", json!({}), json!({ "captions": captions }))
            .duration_in_frames,
        90
    );
    assert_eq!(
        compile_clip(
            "text.captions",
            json!({}),
            json!({ "captions": captions, CLIP: clip_value(&empty_clip(30)) })
        )
        .duration_in_frames,
        30
    );
    let inputs = CardInputs {
        captions_ms: IndexMap::from([("captions".to_owned(), 2500.0)]),
        ..frames(&[("clip", 30)])
    };
    assert!(
        card_texts("text.captions", json!({}), inputs)
            .contains(&"Runs 1.5 s past the clip, which cuts it off".to_owned())
    );
    assert_eq!(
        parse_params("text.captionsFromText", json!({ "script": " " })).unwrap_err(),
        "Write a script"
    );
}
