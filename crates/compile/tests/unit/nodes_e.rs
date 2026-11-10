use indexmap::IndexMap;
use opsis_compile::clip::{
    SequenceSlots, clip_frame_of_recording, empty_clip, recording_props, sequence_clips,
};
use opsis_compile::elements::elements;
use opsis_compile::node::{CardInputs, InputTake};
use opsis_types::cards::Icon;
use opsis_types::timeline::TimelineItem;
use opsis_types::values::{ClipValue, TakeKind};
use serde_json::{Map, Value, json};

use crate::helpers::{
    CLIP, card_texts, clip_value, compile_clip, compile_node, group_value, make_log, node_card,
    problem, style_entries, take_recording, zoomed,
};

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
    let background = |params: Value| {
        let mut all = json!({ "kind": "window" });
        all.as_object_mut()
            .unwrap()
            .extend(params.as_object().unwrap().clone());
        problem("camera.frame", all)
    };
    assert_eq!(
        background(json!({ "background": "solid", "color": "" })).as_deref(),
        Some("Pick a colour")
    );
    assert_eq!(
        background(json!({ "background": "solid", "color": "red" })),
        None
    );
    assert_eq!(
        background(json!({ "background": "gradient", "gradient": " " })).as_deref(),
        Some("Pick a gradient")
    );
    assert_eq!(
        background(json!({ "background": "asset", "asset": "" })).as_deref(),
        Some("Choose an image")
    );
    assert_eq!(background(json!({ "background": "image" })), None);
    assert_eq!(
        problem(
            "camera.frame",
            json!({ "kind": "page", "background": "solid", "color": "" })
        ),
        None
    );
    assert_eq!(
        problem("camera.move", json!({ "fromSpot": true, "moveMs": 0 })).as_deref(),
        Some("Move for must be at least 0.1 s")
    );
    assert_eq!(
        problem(
            "camera.camera",
            json!({ "follows": "cursor", "everyMs": 0 })
        )
        .as_deref(),
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
        None,
    );
    assert_eq!(both.duration_in_frames, 216);
    let frame =
        |ms: f64| clip_frame_of_recording(&both.tracks, &|p| ms / 1000.0 / p.speed * 30.0).unwrap();
    for (ms, expected) in [(3000.0, 60.0), (4000.0, 72.0), (6000.0, 120.0)] {
        assert!((frame(ms) - expected).abs() < 0.01, "{ms}: {}", frame(ms));
    }
    let out = compile_node(
        "camera.speedRamp",
        json!({}),
        json!({ "recording": take_recording(None) }),
    );
    let style = serde_json::to_value(&out["recording"]).unwrap()["edits"].clone();
    assert_eq!(style[0]["type"], "camera.speedRamp");
}

#[test]
fn frames_any_clip_in_a_device_and_draws_window_title_bars() {
    for device in ["laptop", "phone"] {
        let framed = compile_clip(
            "camera.deviceFrame",
            json!({ "device": device }),
            json!({ CLIP: clip_value(&empty_clip(60)) }),
        );
        let wrapper = &group_value(&framed)["wrapper"];
        assert_eq!(wrapper["component"], format!("{device}-frame"));
        let props = wrapper["props"].as_object().unwrap();
        let usual = opsis_compile::elements::element_meta(&format!("{device}-frame"))
            .unwrap()
            .default_props
            .keys()
            .filter(|k| !props.contains_key(*k))
            .filter(|k| !["bezelColor", "screenColor", "fontFamily"].contains(&k.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        assert!(usual.is_empty(), "{device} is missing {usual:?}");
        assert_eq!(props["theme"]["mode"], "dark");
    }
    let phone = compile_clip(
        "camera.deviceFrame",
        json!({ "angle": "tilt", "device": "phone", "tilt": 20, "phone": { "scale": 0.7 } }),
        json!({ CLIP: clip_value(&empty_clip(60)) }),
    );
    let props = &group_value(&phone)["wrapper"]["props"];
    assert_eq!(
        (
            props["variant"].clone(),
            props["tiltAngle"].clone(),
            props["scale"].clone()
        ),
        (json!("tilt"), json!(-20), json!(0.7))
    );
    let laptop = |angle: &str| {
        let framed = compile_clip(
            "camera.deviceFrame",
            json!({ "angle": angle, "tilt": 30 }),
            json!({ CLIP: clip_value(&empty_clip(60)) }),
        );
        group_value(&framed)["wrapper"]["props"]["restTilt"].clone()
    };
    assert_eq!((laptop("flat"), laptop("tilt")), (json!(0), json!(30)));
    for (angle, component) in [
        ("three-d", "screen-3d"),
        ("pull-back", "device-mockup-zoom"),
        ("lineup", "multi-device-lineup"),
    ] {
        let framed = compile_clip(
            "camera.deviceFrame",
            json!({ "angle": angle }),
            json!({ CLIP: clip_value(&empty_clip(45)) }),
        );
        assert_eq!(framed.duration_in_frames, 45, "{angle}");
        assert_eq!(group_value(&framed)["wrapper"]["component"], component);
    }
    let clip = zoomed(
        Some(&[(
            "camera.frame",
            json!({ "windowStyle": "windows", "titleBar": false }),
        )]),
        &[],
        None,
        Some(1.0),
    );
    let props = serde_json::to_value(recording_props(&clip)[0]).unwrap();
    assert_eq!(
        (
            props["window"]["chrome"].clone(),
            props["window"]["style"].clone(),
            props["window"]["bar"].clone()
        ),
        (json!(true), json!("windows"), json!(false))
    );
}

#[test]
fn puts_typed_title_bar_text_in_the_window_frame() {
    let address = |clip: &ClipValue| {
        let props = serde_json::to_value(recording_props(clip)[0]).unwrap();
        (
            props["window"]["text"].clone(),
            props["window"]["mode"].clone(),
        )
    };
    let framed = |params: Value| zoomed(Some(&[("camera.frame", params)]), &[], None, Some(1.0));
    let typed = |mode: &str| (json!("app.example.com"), json!(mode));
    let none = (Value::Null, Value::Null);
    assert_eq!(
        address(&framed(json!({ "title": " app.example.com " }))),
        typed("fixed")
    );
    assert_eq!(
        address(&framed(
            json!({ "title": "app.example.com", "addressMode": "host" })
        )),
        typed("host")
    );
    assert_eq!(
        address(&framed(json!({ "title": "", "addressMode": "host" }))),
        none
    );
    assert_eq!(
        address(&framed(
            json!({ "title": "app.example.com", "titleBar": false })
        )),
        none
    );
    assert_eq!(address(&zoomed(None, &[], None, Some(1.0))), none);
}

#[test]
fn shows_the_typed_title_on_the_frame_card() {
    let frame = |params: Value| card_texts("camera.frame", params, CardInputs::default());
    assert!(frame(json!({})).contains(&"The take's own title".to_owned()));
    assert!(
        frame(json!({ "title": "https://app.example.com/" }))
            .contains(&"app.example.com".to_owned())
    );
    assert!(
        frame(json!({ "title": "app.example.com", "addressMode": "host" }))
            .contains(&"app.example.com · keeps the path".to_owned())
    );
    assert!(
        frame(json!({ "title": "app.example.com", "titleBar": false }))
            .contains(&"No title bar text".to_owned())
    );
}

const fn take(duration_ms: f64, cursor_track: bool) -> InputTake {
    InputTake {
        kind: TakeKind::Browser,
        duration_ms,
        cursor_track,
    }
}

fn takes(port: &str, take: Option<InputTake>, style: &[(&str, Value)]) -> CardInputs {
    CardInputs {
        takes: IndexMap::from([(port.to_owned(), take)]),
        style: IndexMap::from([(
            "style".to_owned(),
            style
                .iter()
                .map(|(kind, params)| opsis_types::values::StyleEntry {
                    node_id: String::new(),
                    kind: (*kind).to_owned(),
                    params: params.as_object().unwrap().clone(),
                })
                .collect(),
        )]),
        ..CardInputs::default()
    }
}

#[test]
fn warns_when_a_camera_move_starts_after_the_take_or_follows_no_cursor() {
    let after = "After the take ends at 6 s".to_owned();
    let moved = |params: Value| {
        card_texts(
            "camera.move",
            params,
            takes("recording", Some(take(6000.0, true)), &[]),
        )
    };
    assert!(moved(json!({ "atMs": 60_000 })).contains(&after));
    assert!(moved(json!({ "atMs": 7000, "fromSpot": true })).contains(&after));
    assert!(!moved(json!({ "atMs": 1000 })).contains(&after));
    let card = |cursor: bool, style: &[(&str, Value)]| {
        card_texts(
            "camera.take",
            json!({}),
            takes("recording", Some(take(6000.0, cursor)), style),
        )
    };
    let follows = [("camera.camera", json!({ "follows": "cursor" }))];
    let warning = "The take has no cursor to follow".to_owned();
    assert!(card(false, &follows).contains(&warning));
    assert!(!card(true, &follows).contains(&warning));
    assert!(!card(false, &[("camera.camera", json!({}))]).contains(&warning));
    let filmed = "The take films its own cursor".to_owned();
    assert!(card(false, &[("cursor.style", json!({}))]).contains(&filmed));
    assert!(!card(true, &[("cursor.style", json!({}))]).contains(&filmed));
}

#[test]
fn lets_the_takes_own_nodes_come_after_the_style() {
    let style = [
        (
            "camera.frame",
            json!({ "background": "solid", "color": "#123456" }),
        ),
        ("camera.camera", json!({ "follows": "cursor" })),
    ];
    let clip = zoomed(
        Some(&style),
        &[("camera.move", json!({ "atMs": 500 }))],
        None,
        Some(1.0),
    );
    let props = serde_json::to_value(recording_props(&clip)[0]).unwrap();
    assert_eq!(
        (
            props["shot"]["backdrop"].clone(),
            props["shot"]["backdropChosen"].clone()
        ),
        (json!("#123456"), json!(true))
    );
    let beats: Vec<Value> = props["extraBeats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["nodeId"].clone())
        .collect();
    assert_eq!(beats, [json!("s1"), json!("t0"), json!("t0")]);
    let plain = zoomed(None, &[], None, Some(1.0));
    assert_eq!(
        serde_json::to_value(recording_props(&plain)[0]).unwrap()["shot"]["backdrop"],
        "glaze"
    );
}

#[test]
fn styles_every_take_in_a_sequence_and_warns_when_a_clip_has_no_recording() {
    let take = || zoomed(None, &[], None, Some(1.0));
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
            .map(|p| p.pointer.scale)
            .collect::<Vec<_>>(),
        [2.0, 2.0]
    );
    let none = "No recording in this clip".to_owned();
    let texts = |t: Option<InputTake>| {
        card_texts(
            "camera.applyStyle",
            json!({}),
            CardInputs {
                takes: IndexMap::from([("clip".to_owned(), t)]),
                ..CardInputs::default()
            },
        )
    };
    assert!(texts(None).contains(&none));
    assert!(
        !texts(Some(InputTake {
            kind: TakeKind::Browser,
            duration_ms: 1000.0,
            cursor_track: true
        }))
        .contains(&none)
    );
}

fn sequencers(tracks: &[opsis_types::timeline::TimelineTrack]) -> Vec<Value> {
    let mut found = Vec::new();
    opsis_compile::clip::walk_items(tracks, &mut |item| {
        if let TimelineItem::Group { props, .. } = item
            && let Some(sequencer) = &props.sequencer
        {
            found.push(serde_json::to_value(sequencer).unwrap());
        }
    });
    found
}

#[test]
fn plays_clips_joined_by_spring_settle_as_scenes_of_one_run() {
    let settle = match compile_node(
        "timeline.transition",
        json!({ "kind": "spring-settle", "durationInFrames": 4, "spring-settle": { "enterScale": 1.3 } }),
        json!({}),
    )
    .shift_remove("transition")
    {
        Some(transition) => serde_json::to_value(transition).unwrap(),
        None => panic!("no transition"),
    };
    let clip = compile_clip(
        "timeline.sequence",
        json!({}),
        json!({
            "in1": clip_value(&empty_clip(30)),
            "in2": clip_value(&empty_clip(20)),
            "transition": settle
        }),
    );
    assert_eq!(clip.duration_in_frames, 54);
    let found = sequencers(&clip.tracks);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["component"], "spring-settle");
    assert_eq!(
        (
            found[0]["props"]["enterScale"].as_f64(),
            found[0]["props"]["gapFrames"].as_f64()
        ),
        (Some(1.3), Some(4.0))
    );
    assert_eq!(found[0]["backgrounds"], json!([null, null]));
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
        "fx.logo-set",
        json!({ "kind": "stack", "stack": { "logos": [{ "image": "assets/a.svg", "background": "#000" }, { "image": "app:placeholders/logo-mark.svg", "background": "#111" }] } }),
    );
    assert_eq!(
        props["logos"],
        json!([{ "image": "project:assets/a.svg", "background": "#000" }, { "image": "app:placeholders/logo-mark.svg", "background": "#111" }])
    );
    assert!(
        !first_item(
            "fx.metric-ticker",
            json!({ "kind": "ring", "ring": { "backgroundColor": "" } })
        )
        .1
        .contains_key("backgroundColor")
    );
    assert_eq!(
        first_item(
            "fx.metric-ticker",
            json!({ "kind": "ring", "ring": { "backgroundColor": "#123456" } })
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
        "fx.background",
        json!({ "kind": "mesh" }),
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
        "fx.text-mask-video",
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
            "fx.text-mask-video",
            json!({ "src": "project:assets/clip.mp4" }),
            Icon::Blend,
            CardInputs::default()
        )
        .is_none()
    );
    assert_eq!(
        row_with(
            "fx.hero-launch",
            json!({}),
            Icon::Moon,
            CardInputs::default()
        )
        .unwrap()
        .text,
        "Dark theme"
    );
    let light = row_with(
        "fx.hero-launch",
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

fn text_leaves(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => out.push(text.clone()),
        Value::Array(items) => items.iter().for_each(|item| text_leaves(item, out)),
        Value::Object(map) => map.values().for_each(|item| text_leaves(item, out)),
        _ => {}
    }
}

#[test]
fn no_element_default_loads_from_the_internet_or_names_a_library() {
    let banned = regex::Regex::new(
        r"(?i)https?://|\b(playwright|chromium|ffmpeg|lottie|three\.js|react|tailwind|shadcn|framer|gsap|remocn|snapcn|fontsource)\b",
    )
    .unwrap();
    let found: Vec<String> = elements()
        .iter()
        .flat_map(|meta| {
            let mut texts = Vec::new();
            text_leaves(
                &serde_json::to_value(&meta.default_props).unwrap(),
                &mut texts,
            );
            text_leaves(
                &serde_json::to_value(&meta.fixed_props).unwrap(),
                &mut texts,
            );
            texts
                .into_iter()
                .filter(|text| banned.is_match(text))
                .map(move |text| format!("{}: {text}", meta.key))
        })
        .collect();
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn passes_element_times_in_frames_of_thirty_a_second_and_lays_out_fixed_size_elements() {
    assert_eq!(
        first_item("fx.status-cycle", json!({ "statusHold": 1.5 })).1["statusHold"].as_f64(),
        Some(45.0)
    );
    assert_eq!(
        first_item("fx.punch-lines", json!({ "holds": "1, 2.5" })).1["holds"],
        "30, 75"
    );
    let steps = first_item(
        "fx.keystroke",
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
        first_item("fx.chart", json!({ "kind": "funnel" })).2,
        Some(json!({ "width": 1920, "height": 1080 }))
    );
    assert_eq!(
        first_item("fx.animated-text", json!({ "soft-blur": { "text": "Hi" } })).2,
        None
    );
}

#[test]
fn speed_lines_are_an_element_of_their_own_with_no_clip_inside() {
    let clip = compile_clip("fx.speed-lines", json!({}), json!({}));
    let item = serde_json::to_value(&clip.tracks[0].items[0]).unwrap();
    assert_eq!(item["props"]["children"], Value::Null);
    assert!(item["props"].as_object().unwrap().contains_key("children"));
}

#[test]
fn a_text_mask_fills_with_the_kind_of_file_it_is_given() {
    let media = |src: &str| {
        first_item("fx.text-mask-video", json!({ "src": src }))
            .1
            .get("media")
            .cloned()
    };
    assert_eq!(media("project:assets/shot.png"), Some(json!("image")));
    assert_eq!(media("project:assets/clip.mp4"), Some(json!("video")));
    assert_eq!(media(""), None);
}

#[test]
fn the_inspector_knows_the_kind_of_each_file_setting() {
    use opsis_compile::compile::compile_workflow;
    use opsis_compile::inspector::inspector_context;
    use opsis_compile::nodes::media::IMPORT_TYPE;
    use opsis_types::values::MediaKind;
    let kinds = |path: &str| {
        let mut g = crate::helpers::Graph::default();
        g.node("file", IMPORT_TYPE, json!({ "path": path }));
        let wf = g.build();
        let compiled = compile_workflow(&wf, None);
        inspector_context(&wf, &compiled, &IndexMap::new(), "file")
            .unwrap()
            .file_kinds
    };
    assert_eq!(
        kinds("project:assets/shot.png").get("path"),
        Some(&MediaKind::Image)
    );
    assert_eq!(
        kinds("project:assets/talk.wav").get("path"),
        Some(&MediaKind::Audio)
    );
    assert!(kinds("").is_empty());
}

#[test]
fn says_on_record_takes_card_when_the_take_is_smooth() {
    let first =
        |params: Value| card_texts("capture.record", params, CardInputs::default())[0].clone();
    assert_eq!(first(json!({})), "30 fps");
    assert_eq!(
        first(json!({ "timing": "smooth", "fps": 60 })),
        "60 fps · Smooth"
    );
}
