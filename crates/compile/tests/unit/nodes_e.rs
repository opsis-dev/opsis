use opsis_compile::clip::{
    SequenceSlots, clip_frame_of_recording, empty_clip, recording_props, sequence_clips,
};
use opsis_compile::elements::elements;
use opsis_compile::node::{CardInputs, InputTake};
use opsis_compile::registry::node_def;
use opsis_types::cards::{Icon, Tone};
use opsis_types::timeline::TimelineItem;
use opsis_types::values::ClipValue;
use serde_json::{Map, Value, json};

use crate::helpers::{
    CLIP, card_texts, clip_value, compile_clip, compile_node, make_log, node_card, parse_params,
    style_entries, take_recording, zoomed,
};

fn problem(kind: &str, params: Value) -> Option<String> {
    parse_params(kind, params).err()
}

#[test]
fn keeps_a_crop_inside_the_recording_and_asks_backdrops_for_what_their_kind_needs() {
    assert_eq!(
        problem(
            "camera.crop",
            json!({ "x": 0, "y": 0, "w": 0.5, "h": 0.25 })
        ),
        None
    );
    assert_eq!(
        problem(
            "camera.crop",
            json!({ "x": 0.8, "y": 0, "w": 0.5, "h": 0.5 })
        )
        .as_deref(),
        Some("The area runs past the right edge")
    );
    assert_eq!(
        problem(
            "camera.crop",
            json!({ "x": 0, "y": 0.6, "w": 0.5, "h": 0.5 })
        )
        .as_deref(),
        Some("The area runs past the bottom edge")
    );
    assert_eq!(
        problem("camera.crop", json!({ "x": 0, "y": 0, "w": 0, "h": 0.5 })).as_deref(),
        Some("Width must be at least 0.05")
    );
    assert_eq!(
        problem("camera.backdrop", json!({ "kind": "solid", "color": "" })).as_deref(),
        Some("Pick a colour")
    );
    assert_eq!(
        problem(
            "camera.backdrop",
            json!({ "kind": "solid", "color": "red" })
        ),
        None
    );
    assert_eq!(
        problem(
            "camera.backdrop",
            json!({ "kind": "gradient", "gradient": " " })
        )
        .as_deref(),
        Some("Pick a gradient")
    );
    assert_eq!(
        problem("camera.backdrop", json!({ "kind": "asset", "asset": "" })).as_deref(),
        Some("Choose an image")
    );
    assert_eq!(problem("camera.backdrop", json!({ "kind": "image" })), None);
    assert_eq!(
        problem("camera.manualZoom", json!({ "holdMs": 0 })).as_deref(),
        Some("Hold must be at least 0.1 s")
    );
    assert_eq!(
        problem("camera.pan", json!({ "durationMs": 0 })).as_deref(),
        Some("Pan for must be at least 0.1 s")
    );
    assert_eq!(
        problem("camera.followCursor", json!({ "everyMs": 0 })).as_deref(),
        Some("Re-aim every must be at least 0.2 s")
    );
}

#[test]
fn places_a_second_speed_ramp_at_its_own_recording_time() {
    let ramp = |from: f64, to: f64| {
        (
            "camera.speedRamp",
            json!({ "segments": [{ "fromMs": from, "toMs": to, "rate": 2 }] }),
        )
    };
    let both = zoomed(
        None,
        &[ramp(1000.0, 2000.0), ramp(3000.0, 4000.0)],
        Some(make_log(Vec::new(), 10_000.0)),
        json!({ "speed": 1 }),
    );
    assert_eq!(both.duration_in_frames, 270.0);
    let frame = |ms: f64| clip_frame_of_recording(&both.tracks, &|_| ms / 1000.0 * 30.0).unwrap();
    for (ms, expected) in [(3000.0, 75.0), (4000.0, 90.0), (6000.0, 150.0)] {
        assert!((frame(ms) - expected).abs() < 0.01, "{ms}: {}", frame(ms));
    }
    let out = compile_node(
        "camera.speedRamp",
        json!({}),
        json!({ "recording": take_recording(None) }),
    );
    let style = serde_json::to_value(&out["recording"]).unwrap()["style"].clone();
    assert_eq!(style[0]["type"], "camera.speedRamp");
}

fn group_value(clip: &ClipValue) -> Value {
    match &clip.tracks[0].items[0] {
        TimelineItem::Group { props, .. } => serde_json::to_value(props).unwrap(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn frames_any_clip_in_a_device_and_draws_window_title_bars() {
    for device in ["laptop", "phone"] {
        let framed = compile_clip(
            "camera.deviceFrame",
            json!({ "device": device }),
            json!({ CLIP: clip_value(&empty_clip(60.0)) }),
        );
        assert_eq!(
            group_value(&framed)["wrapper"]["component"],
            format!("snapcn/{device}-frame")
        );
    }
    let clip = zoomed(
        Some(&[(
            "camera.windowFrame",
            json!({ "windowStyle": "windows", "addressBar": false }),
        )]),
        &[],
        None,
        json!({ "speed": 1 }),
    );
    let props = serde_json::to_value(recording_props(&clip)[0]).unwrap();
    assert_eq!(
        (
            props["chrome"].clone(),
            props["windowStyle"].clone(),
            props["addressBar"].clone()
        ),
        (json!(true), json!("windows"), json!(false))
    );
}

fn take(duration_ms: f64, cursor_track: bool) -> InputTake {
    InputTake {
        duration_ms,
        cursor_track,
    }
}

fn takes(port: &str, take: Option<InputTake>, style: &[&str]) -> CardInputs {
    CardInputs {
        takes: [(port.to_owned(), take)].into_iter().collect(),
        style: [(
            "style".to_owned(),
            style.iter().map(|s| (*s).to_owned()).collect(),
        )]
        .into_iter()
        .collect(),
        ..CardInputs::default()
    }
}

#[test]
fn warns_when_a_camera_move_starts_after_the_take_or_follows_no_cursor() {
    let after = "After the take ends at 6 s".to_owned();
    assert!(
        card_texts(
            "camera.manualZoom",
            json!({ "atMs": 60_000 }),
            takes("recording", Some(take(6000.0, true)), &[])
        )
        .contains(&after)
    );
    assert!(
        card_texts(
            "camera.pan",
            json!({ "atMs": 7000 }),
            takes("recording", Some(take(6000.0, true)), &[])
        )
        .contains(&after)
    );
    assert!(
        !card_texts(
            "camera.manualZoom",
            json!({ "atMs": 1000 }),
            takes("recording", Some(take(6000.0, true)), &[])
        )
        .contains(&after)
    );
    let card = |cursor: bool, style: &[&str]| {
        card_texts(
            "camera.autoZoom",
            json!({}),
            takes("recording", Some(take(6000.0, cursor)), style),
        )
    };
    let warning = "The take has no cursor to follow".to_owned();
    assert!(card(false, &["camera.followCursor"]).contains(&warning));
    assert!(!card(true, &["camera.followCursor"]).contains(&warning));
    assert!(!card(false, &["camera.backdrop"]).contains(&warning));
    let filmed = "The take films its own cursor".to_owned();
    assert!(card(false, &["cursor.style"]).contains(&filmed));
    assert!(!card(true, &["cursor.style"]).contains(&filmed));
}

#[test]
fn lets_the_style_beat_auto_zoom_and_the_takes_own_nodes_come_last() {
    let style = [
        (
            "camera.backdrop",
            json!({ "kind": "solid", "color": "#123456" }),
        ),
        ("camera.followCursor", json!({})),
    ];
    let clip = zoomed(
        Some(&style),
        &[("camera.manualZoom", json!({ "atMs": 500 }))],
        None,
        json!({ "speed": 1, "backdrop": "aurora" }),
    );
    let props = serde_json::to_value(recording_props(&clip)[0]).unwrap();
    assert_eq!(
        (props["backdrop"].clone(), props["backdropChosen"].clone()),
        (json!("#123456"), json!(true))
    );
    let beats: Vec<Value> = props["extraBeats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["nodeId"].clone())
        .collect();
    assert_eq!(beats, [json!("s1"), json!("t0"), json!("t0")]);
    let plain = zoomed(None, &[], None, json!({ "speed": 1, "backdrop": "aurora" }));
    assert_eq!(
        serde_json::to_value(recording_props(&plain)[0]).unwrap()["backdrop"],
        "aurora"
    );
}

#[test]
fn styles_every_take_in_a_sequence_and_warns_when_a_clip_has_no_recording() {
    let take = || zoomed(None, &[], None, json!({ "speed": 1 }));
    let sequence = sequence_clips(
        "seq",
        &[take(), take()],
        &[],
        SequenceSlots {
            ports: None,
            slot_node: None,
        },
    );
    let style = json!({ "type": "recording-style", "entries": style_entries(&[("cursor.style", json!({ "size": 2 }))], "s") });
    let styled = compile_clip(
        "camera.applyStyle",
        json!({}),
        json!({ CLIP: clip_value(&sequence), "style": style }),
    );
    assert_eq!(
        recording_props(&styled)
            .iter()
            .map(|p| p.cursor_scale)
            .collect::<Vec<_>>(),
        [2.0, 2.0]
    );
    let none = "No recording in this clip".to_owned();
    let texts = |t: Option<InputTake>| {
        card_texts(
            "camera.applyStyle",
            json!({}),
            CardInputs {
                takes: [("clip".to_owned(), t)].into_iter().collect(),
                ..CardInputs::default()
            },
        )
    };
    assert!(texts(None).contains(&none));
    assert!(
        !texts(Some(InputTake {
            duration_ms: 1000.0,
            cursor_track: true
        }))
        .contains(&none)
    );
}

const SPRING_SETTLE: &str = "fx.remocn.spring-settle";

#[test]
fn plays_connected_clips_as_spring_settle_scenes_and_counts_them_on_its_card() {
    let clip = compile_clip(
        SPRING_SETTLE,
        json!({ "gapFrames": 4, "enterScale": 1.3, "backgrounds": [{ "scene": "in3", "color": "#112233" }] }),
        json!({ "in1": clip_value(&empty_clip(30.0)), "in3": clip_value(&empty_clip(20.0)) }),
    );
    assert_eq!(clip.duration_in_frames, 54.0);
    let sequencer = group_value(&clip)["sequencer"].clone();
    assert_eq!(sequencer["component"], "remocn/spring-settle");
    assert_eq!(
        (
            sequencer["props"]["enterScale"].as_f64(),
            sequencer["props"]["gapFrames"].as_f64()
        ),
        (Some(1.3), Some(4.0))
    );
    assert_eq!(sequencer["backgrounds"], json!([null, "#112233"]));
    let inputs = CardInputs {
        connected: vec!["in1".into(), "in2".into()],
        items: [("in1".to_owned(), 1.0), ("in2".to_owned(), 3.0)]
            .into_iter()
            .collect(),
        clip_frames: Some(90.0),
        ..CardInputs::default()
    };
    let rows = card_texts(SPRING_SETTLE, json!({ "gapFrames": 15 }), inputs);
    assert_eq!(
        (rows[0].as_str(), rows[1].as_str()),
        ("2 scenes", "Items per scene: 1 · 3")
    );
    assert!(rows.iter().any(|t| t.ends_with("between scenes")));
    let empty = node_card(SPRING_SETTLE, json!({}), CardInputs::default())[0].clone();
    assert_eq!(
        (empty.text.as_str(), empty.tone),
        ("0 scenes", Some(Tone::Warn))
    );
}

fn first_item(kind: &str, params: Value) -> (String, Map<String, Value>, Option<Value>) {
    let clip = compile_clip(kind, params, json!({}));
    match &clip.tracks[0].items[0] {
        TimelineItem::Element {
            component,
            props,
            base,
        } => (
            component.clone(),
            props.clone(),
            base.design
                .as_ref()
                .map(|d| serde_json::to_value(d).unwrap()),
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn makes_files_in_rows_project_files_and_drops_colours_left_on_the_theme() {
    let (_, props, _) = first_item(
        "fx.remocn.logo-enter",
        json!({ "logos": [{ "image": "assets/a.svg", "background": "#000" }, { "image": "app:placeholders/logo-mark.svg", "background": "#111" }] }),
    );
    assert_eq!(
        props["logos"],
        json!([{ "image": "project:assets/a.svg", "background": "#000" }, { "image": "app:placeholders/logo-mark.svg", "background": "#111" }])
    );
    assert!(
        !first_item("fx.remotion-ui.stat-card", json!({ "backgroundColor": "" }))
            .1
            .contains_key("backgroundColor")
    );
    assert_eq!(
        first_item(
            "fx.remotion-ui.stat-card",
            json!({ "backgroundColor": "#123456" })
        )
        .1["backgroundColor"],
        "#123456"
    );
}

fn row_with(
    kind: &str,
    params: Value,
    icon: Icon,
    inputs: CardInputs,
) -> Option<opsis_types::cards::CardRow> {
    node_card(kind, params, inputs)
        .into_iter()
        .find(|r| r.icon == icon)
}

#[test]
fn shows_palettes_gradients_and_themes_on_element_cards() {
    let palette = row_with(
        "fx.remocn.shader-mesh-gradient",
        json!({}),
        Icon::Palette,
        CardInputs::default(),
    )
    .unwrap();
    assert_eq!(
        (palette.text.as_str(), palette.swatches.unwrap()),
        (
            "4 colours",
            vec![
                "#12121a".to_owned(),
                "#232338".into(),
                "#3a3a5c".into(),
                "#52527a".into()
            ]
        )
    );
    let gradient = row_with(
        "fx.remotion-ui.text-mask-video",
        json!({}),
        Icon::Blend,
        CardInputs::default(),
    )
    .unwrap();
    assert_eq!(
        (gradient.text.as_str(), gradient.swatches.unwrap()),
        (
            "Straight gradient",
            vec!["#e8b86d".to_owned(), "#f472b6".into(), "#2dd4bf".into()]
        )
    );
    assert!(
        row_with(
            "fx.remotion-ui.text-mask-video",
            json!({ "src": "project:assets/clip.mp4" }),
            Icon::Blend,
            CardInputs::default()
        )
        .is_none()
    );
    assert_eq!(
        row_with(
            "fx.snapcn.hero-launch",
            json!({}),
            Icon::Moon,
            CardInputs::default()
        )
        .unwrap()
        .text,
        "Dark theme"
    );
    let light = row_with(
        "fx.snapcn.hero-launch",
        json!({ "theme": { "mode": "light", "colors": { "background": "#fff" } } }),
        Icon::Sun,
        CardInputs::default(),
    )
    .unwrap();
    assert_eq!(
        (light.text.as_str(), light.swatches.unwrap()),
        ("Light theme · 1 colour changed", vec!["#fff".to_owned()])
    );
}

#[test]
fn every_elements_defaults_pass_its_own_schema() {
    let failing: Vec<String> = elements()
        .iter()
        .filter_map(|meta| {
            meta.schema
                .parse(&meta.default_props, None)
                .err()
                .map(|e| format!("{}: {e}", meta.key))
        })
        .collect();
    assert!(failing.is_empty(), "{failing:?}");
}

#[test]
fn plays_element_music_through_the_mix_as_the_nodes_own_channel() {
    let clip = compile_clip(
        "fx.remocn.release-teaser",
        json!({ "music": "assets/song.mp3", "musicVolume": 0.4 }),
        json!({}),
    );
    assert_eq!(clip.audio.len(), 1);
    let sound = serde_json::to_value(&clip.audio[0]).unwrap();
    assert_eq!(
        (
            sound["src"].clone(),
            sound["volume"].clone(),
            sound["channel"].clone(),
            sound["role"].clone()
        ),
        (
            json!("project:assets/song.mp3"),
            json!(0.4),
            json!("n"),
            json!("music")
        )
    );
    assert_eq!(
        sound["durationInFrames"].as_f64(),
        Some(clip.duration_in_frames)
    );
    if let TimelineItem::Element { props, .. } = &clip.tracks[0].items[0] {
        assert!(!props.contains_key("music") && !props.contains_key("musicVolume"));
    }
    assert!(
        compile_clip("fx.remocn.release-teaser", json!({}), json!({}))
            .audio
            .is_empty()
    );
}

#[test]
fn passes_element_times_in_frames_of_thirty_a_second_and_lays_out_fixed_size_elements() {
    assert_eq!(
        first_item("fx.snapcn.status-cycle", json!({ "statusHold": 1.5 })).1["statusHold"].as_f64(),
        Some(45.0)
    );
    assert_eq!(
        first_item("fx.snapcn.punch-lines", json!({ "holds": "1, 2.5" })).1["holds"],
        "30, 75"
    );
    let steps = first_item(
        "fx.remocn.keystroke",
        json!({ "steps": [{ "keys": "mod+k", "at": 2, "hold": 0.5 }] }),
    )
    .1["steps"]
        .clone();
    assert_eq!(
        (
            steps[0]["keys"].clone(),
            steps[0]["at"].as_f64(),
            steps[0]["hold"].as_f64()
        ),
        (json!("mod+k"), Some(60.0), Some(15.0))
    );
    assert_eq!(
        first_item("fx.remotion-ui.funnel-chart", json!({})).2,
        Some(json!({ "width": 1920.0, "height": 1080.0 }))
    );
    assert_eq!(
        first_item("fx.remocn.soft-blur-in", json!({ "text": "Hi" })).2,
        None
    );
}

const DEVICE_MOCKUP: &str = "fx.remotion-ui.device-mockup-zoom";

#[test]
fn runs_a_device_mockup_with_or_without_a_clip() {
    assert!(
        node_def(DEVICE_MOCKUP)
            .unwrap()
            .input("clip")
            .unwrap()
            .optional
    );
    let (component, props, _) =
        first_item(DEVICE_MOCKUP, json!({ "src": "project:assets/screen.png" }));
    assert_eq!(
        (component.as_str(), props["src"].clone()),
        (
            "remotion-ui/device-mockup-zoom",
            json!("project:assets/screen.png")
        )
    );
    let field = node_def(DEVICE_MOCKUP)
        .unwrap()
        .params
        .field("clipDuration")
        .unwrap()
        .clone();
    assert_eq!(
        (
            field["x-zero-label"].clone(),
            field["x-unconnected-zero-label"].clone()
        ),
        (json!("Match the clip"), json!("Usual length"))
    );
    let clock = |kind: &str, connected: &[&str]| {
        row_with(
            kind,
            json!({}),
            Icon::Clock,
            CardInputs {
                connected: connected.iter().map(|c| (*c).to_owned()).collect(),
                ..CardInputs::default()
            },
        )
        .map(|r| r.text)
    };
    assert_eq!(
        clock(DEVICE_MOCKUP, &["clip"]).as_deref(),
        Some("Matches the clip")
    );
    assert_eq!(clock(DEVICE_MOCKUP, &[]).as_deref(), Some("3 s"));
    assert_eq!(
        clock("fx.remotion-ui.multi-device-lineup", &[]).as_deref(),
        Some("Matches the clip")
    );
    let clip = compile_clip(
        DEVICE_MOCKUP,
        json!({}),
        json!({ CLIP: clip_value(&empty_clip(45.0)) }),
    );
    assert_eq!(clip.duration_in_frames, 45.0);
    assert_eq!(
        group_value(&clip)["wrapper"]["component"],
        "remotion-ui/device-mockup-zoom"
    );
}
