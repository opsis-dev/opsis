use opsis_compile::audio::sound_length;
use opsis_compile::clip::{ItemExtra, empty_clip, item_clip, recording_props, walk_items};
use opsis_compile::elements::core::CAPTIONS_ELEMENT;
use opsis_compile::elements::elements;
use opsis_compile::elements::introspect::FieldWidget;
use opsis_compile::node::{CardInputs, InputMedia, PortSpec};
use opsis_compile::nodes::text::{ScriptPace, ends_sentence, script_to_captions};
use opsis_compile::nodes::transitions::presentations;
use opsis_compile::ports::{can_connect, convert_input};
use opsis_compile::registry::registry;
use opsis_compile::sounds::{BuiltinSong, BuiltinSound, Mood, builtin_songs, builtin_sounds};
use opsis_types::cards::Tone;
use opsis_types::numbers::whole;
use opsis_types::timeline::TimelineItem;
use opsis_types::values::{CompileValue, MediaKind};
use opsis_types::workflow::PortType;
use serde_json::{Map, Value, json};

use crate::helpers::{
    CLIP, card_texts, clip_value, compile_clip, compile_node, compile_node_with, node_card,
    node_card_with, node_result, parse_params, typed, zoomed,
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
        json!({ "type": "recording", "src": "project:take.mp4", "log": null, "durationMs": 4000, "viewport": { "width": 1280, "height": 720 }, "nodeId": "rec", "style": [] }),
    )
}

fn spec(kind: PortType, label: Option<&'static str>) -> PortSpec {
    PortSpec {
        kind,
        label: label.map(Into::into),
        optional: false,
        many: false,
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
        (PortType::FileInfo, PortType::Media),
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
        typed(json!({ "type": "image", "src": "project:s.png", "width": 3, "height": 2 }));
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
        .map(|m| format!("transition.{}", m.key.replacen('/', ".", 1)))
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
        Some(CompileValue::Audio(audio)) => assert!(audio.items.is_empty()),
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
            media: [("media".to_owned(), InputMedia { kind, has_audio })]
                .into_iter()
                .collect(),
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
        media: [(
            "media".to_owned(),
            InputMedia {
                kind: MediaKind::Audio,
                has_audio: Some(true),
            },
        )]
        .into_iter()
        .collect(),
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

fn blurred(params: &[Value]) -> Value {
    let own: Vec<(&str, Value)> = params
        .iter()
        .map(|p| ("effect.blurAreas", p.clone()))
        .collect();
    serde_json::to_value(recording_props(&zoomed(None, &own, None, json!({ "speed": 1 })))[0])
        .unwrap()
}

#[test]
fn puts_blur_areas_on_the_take_and_adds_them_up_across_nodes() {
    let props = blurred(&[json!({ "strength": 0.5, "softness": 0, "roundness": 0.2 })]);
    assert_eq!(
        props["blurAreas"],
        json!([{ "x": 0.62, "y": 0.06, "width": 0.3, "height": 0.08, "strength": 0.02, "softness": 0.0, "roundness": 0.2 }])
    );
    let props = blurred(&[
        json!({ "fromMs": 1000, "untilMs": 2000 }),
        json!({ "areas": [{ "x": 0, "y": 0, "width": 0.1, "height": 0.1 }], "fromMs": 3000 }),
    ]);
    let times: Vec<(Value, Value)> = props["blurAreas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| (a["fromMs"].clone(), a["untilMs"].clone()))
        .collect();
    assert_eq!(
        times,
        [(json!(1000.0), json!(2000.0)), (json!(3000.0), Value::Null)]
    );
    assert_eq!(
        parse_params(
            "effect.blurAreas",
            json!({ "fromMs": 2000, "untilMs": 1000 })
        )
        .unwrap_err(),
        "Until must come after From"
    );
    assert_eq!(
        card_texts(
            "effect.blurAreas",
            json!({ "fromMs": 500 }),
            CardInputs::default()
        ),
        ["1 area · 60% strength", "From 0.5 s"]
    );
    assert_eq!(
        card_texts(
            "effect.blurAreas",
            json!({ "areas": [] }),
            CardInputs::default()
        )[0],
        "Draw an area to blur"
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

#[test]
fn gives_a_picture_look_only_to_recordings_videos_and_images_and_names_it() {
    let look = json!({ "warmth": 0.3, "grain": 0.2 });
    let take = zoomed(None, &[], None, json!({ "speed": 1 }));
    let rec = compile_clip(
        "effect.look",
        look.clone(),
        json!({ CLIP: clip_value(&take) }),
    );
    assert_eq!(
        serde_json::to_value(recording_props(&rec)[0]).unwrap()["look"],
        look
    );
    for (id, component, src) in [
        ("v", "core/video", "project:a.mp4"),
        ("i", "core/image", "project:a.png"),
    ] {
        let clip = item_clip(
            id,
            component,
            json!({ "src": src }).as_object().unwrap().clone(),
            30,
            ItemExtra::default(),
        );
        let looked = compile_clip(
            "effect.look",
            look.clone(),
            json!({ CLIP: clip_value(&clip) }),
        );
        assert_eq!(element_props(&looked)[0]["look"], look, "{component}");
    }
    let title = item_clip(
        "t",
        "core/title",
        json!({ "title": "Hi" }).as_object().unwrap().clone(),
        30,
        ItemExtra::default(),
    );
    assert!(
        !element_props(&compile_clip(
            "effect.look",
            json!({ "warmth": 0.3 }),
            json!({ CLIP: clip_value(&title) })
        ))[0]
            .contains_key("look")
    );
    let natural = node_card("effect.look", json!({}), CardInputs::default())[0].clone();
    assert_eq!(
        (natural.text.as_str(), natural.tone),
        ("Natural", Some(Tone::Muted))
    );
    assert_eq!(
        card_texts(
            "effect.look",
            json!({ "saturation": 0, "contrast": 1.15 }),
            CardInputs::default()
        )[0],
        "Black and white"
    );
    assert_eq!(
        card_texts(
            "effect.look",
            json!({ "grain": 0.1 }),
            CardInputs::default()
        )[0],
        "1 change"
    );
}

fn pace(words_per_minute: f64, start_ms: f64, line_pause_ms: f64) -> ScriptPace {
    ScriptPace {
        words_per_minute,
        start_ms,
        line_pause_ms,
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
    assert!(script_to_captions("  ", pace(160.0, 0.0, 0.0)).is_empty());
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
        captions_ms: [("captions".to_owned(), 2500.0)].into_iter().collect(),
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
