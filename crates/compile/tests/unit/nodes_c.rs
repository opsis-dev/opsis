use opsis_compile::card::{element_row, sentence};
use opsis_compile::compile::interpolated_params;
use opsis_compile::node::{CardCtx, CardInputs, MAX_CARD_ROWS, NodeDef};
use opsis_compile::registry::{node_def, registry};
use opsis_compile::vars::Vars;
use opsis_types::cards::{Icon, Tone};
use opsis_types::timeline::TimelineItem;
use opsis_types::workflow::WorkflowSettings;
use regex::Regex;
use serde_json::{Map, Value, json};

use crate::helpers::{
    action, card_texts, compile_clip, make_log, node_result, parse_params, record_result,
};

fn element_of(kind: &str, params: Value, inputs: Value) -> (String, Map<String, Value>, u32) {
    let clip = compile_clip(kind, params, inputs);
    match &clip.tracks[0].items[0] {
        TimelineItem::Element {
            component, props, ..
        } => (component.clone(), props.clone(), clip.duration_in_frames),
        other => panic!("{other:?}"),
    }
}

fn media(kind: &str, src: &str, has_audio: Option<bool>) -> Value {
    let mut m = json!({ "type": "media", "src": src, "kind": kind });
    if let Some(a) = has_audio {
        m["hasAudio"] = json!(a);
    }
    m
}

#[test]
fn shows_sound_files_as_placeholders_mutes_silent_videos_and_gives_images_one_length() {
    let sound = json!({ "media": media("audio", "project:assets/voice.mp3", None) });
    assert_eq!(
        element_of("media.toClip", json!({}), sound).0,
        "core/placeholder"
    );
    assert_eq!(
        element_of(
            "media.import",
            json!({ "path": "assets/voice.mp3" }),
            json!({})
        )
        .0,
        "core/placeholder"
    );
    let silent = json!({ "media": media("video", "project:out/film.mp4", Some(false)) });
    let loud = json!({ "media": media("video", "project:out/film.mp4", Some(true)) });
    assert_eq!(
        element_of("media.toClip", json!({ "muted": false }), silent).1["muted"],
        true
    );
    assert_eq!(
        element_of("media.toClip", json!({ "muted": false }), loud).1["muted"],
        false
    );
    let image = json!({ "media": media("image", "project:assets/shot.png", None) });
    assert_eq!(
        element_of("media.toClip", json!({}), image).2,
        element_of(
            "media.import",
            json!({ "path": "assets/shot.png" }),
            json!({})
        )
        .2
    );
}

#[test]
fn checks_trim_and_media_file_settings() {
    let problem = |params: Value| parse_params("media.trim", params).err();
    assert_eq!(
        problem(json!({ "startMs": 2000, "endMs": 1000 })).as_deref(),
        Some("End must come after Start")
    );
    assert_eq!(
        problem(json!({ "startMs": 2000, "endMs": 2000 })).as_deref(),
        Some("End must come after Start")
    );
    assert_eq!(problem(json!({ "startMs": 2000, "endMs": 0 })), None);
    assert_eq!(
        problem(json!({ "startMs": 700_000, "endMs": 800_000 })),
        None
    );
    assert_eq!(
        parse_params("media.import", json!({ "path": "a.png", "stillMs": 0 })).unwrap_err(),
        "Length must be at least 0.1 s"
    );
}

#[test]
fn sums_up_media_nodes_on_their_cards() {
    let texts = |codec: &str| {
        card_texts(
            "media.convert",
            json!({ "codec": codec }),
            CardInputs::default(),
        )
    };
    assert_eq!(texts("h264"), ["MP4 · High quality"]);
    assert_eq!(texts("vp9"), ["WebM · High quality"]);
    assert_eq!(texts("prores"), ["ProRes · no Preview"]);
    assert_eq!(
        texts("h265"),
        ["MP4 (HEVC) · High quality · may not preview"]
    );
    let join = |connected: &[&str]| {
        card_texts(
            "media.concat",
            json!({}),
            CardInputs {
                connected: connected.iter().map(|c| (*c).to_owned()).collect(),
                ..CardInputs::default()
            },
        )[0]
        .clone()
    };
    assert_eq!(
        [join(&[]), join(&["in1"]), join(&["in1", "in2", "in3"])],
        ["Connect two files", "Connect one more file", "3 files"]
    );
    let cards = [
        (
            "media.import",
            json!({ "path": "assets/a.mp4" }),
            vec!["a.mp4"],
        ),
        (
            "media.trim",
            json!({ "startMs": 1000, "endMs": 4000 }),
            vec!["1 s → 4 s"],
        ),
        (
            "media.speed",
            json!({ "rate": 2, "keepAudio": false }),
            vec!["2×", "No sound"],
        ),
        (
            "media.gif",
            json!({ "width": 640, "fps": 12, "loop": false }),
            vec!["640 px · 12 fps"],
        ),
        (
            "media.extractAudio",
            json!({ "format": "wav" }),
            vec!["WAV"],
        ),
        (
            "media.thumbnail",
            json!({ "atMs": 2500, "width": 640 }),
            vec!["At 2.5 s · 640 px"],
        ),
    ];
    for (kind, params, expected) in cards {
        assert_eq!(
            card_texts(kind, params, CardInputs::default()),
            expected,
            "{kind}"
        );
    }
    let scaled = card_texts(
        "media.scalePad",
        json!({ "width": 1080, "height": 1920, "mode": "crop" }),
        CardInputs::default(),
    );
    assert_eq!((scaled.len(), scaled[0].as_str()), (2, "1080 × 1920"));
}

#[test]
fn fills_a_variable_into_a_file_path_before_reading_the_file() {
    let def = node_def("media.import").unwrap();
    let vars: Vars = [("Folder".to_owned(), "assets/clips".to_owned())]
        .into_iter()
        .collect();
    let params = json!({ "path": "{{Folder}}/intro.mp4" })
        .as_object()
        .unwrap()
        .clone();
    let values = def
        .parse(
            &interpolated_params(Some(def), &params, &vars),
            Some(&params),
        )
        .unwrap();
    assert_eq!(values["path"], "assets/clips/intro.mp4");
}

#[test]
fn describes_elements_in_words_and_titles_in_sentence_case() {
    let named = |target: &str, role: Option<&str>| element_row(Icon::Crosshair, target, role).text;
    assert_eq!(named("Login", Some("button")), "“Login” button");
    assert_eq!(named("Username", Some("textbox")), "“Username” field");
    assert_eq!(
        named("What needs to be done?", None),
        "“What needs to be done?”"
    );
    assert_eq!(named("css=#add-to-cart", None), "Custom element");
    assert_eq!(named("320, 180", None), "Point 320, 180");
    let empty = element_row(Icon::Crosshair, "  ", None);
    assert_eq!(
        (empty.text.as_str(), empty.tone),
        ("Pick an element", Some(Tone::Warn))
    );
    assert_eq!(
        [
            sentence("Whip Pan"),
            sentence("RGB Glitch Text"),
            sentence("Slide In 3D")
        ],
        ["Whip pan", "RGB glitch text", "Slide in 3D"]
    );
}

fn rows_for(
    def: &NodeDef,
    values: &Map<String, Value>,
    inputs: &CardInputs,
    result: Option<&opsis_types::values::NodeResult>,
) -> Vec<opsis_types::cards::CardRow> {
    let settings = WorkflowSettings::default();
    let values = def.params.card_values(values, &Vars::new());
    def.all_rows(&CardCtx {
        params: (),
        values: &values,
        settings: &settings,
        result,
        inputs,
    })
}

fn all_ports(def: &NodeDef) -> Vec<String> {
    def.inputs.iter().map(|(p, _)| (*p).to_owned()).collect()
}

#[test]
fn shows_a_well_formed_row_on_every_visible_card_from_default_settings() {
    let result = node_result(
        json!({ "outputs": { "media": { "type": "media", "src": null, "kind": "video", "durationMs": 1000 } } }),
    );
    let empty: Vec<String> = registry()
        .visible()
        .filter(|def| {
            let inputs = CardInputs {
                connected: all_ports(def),
                ..CardInputs::default()
            };
            let rows = rows_for(def, &def.defaults(), &inputs, Some(&result));
            rows.is_empty() || rows.iter().any(|r| r.text.is_empty())
        })
        .map(|d| d.kind.to_string())
        .collect();
    assert!(empty.is_empty(), "{empty:?}");
}

fn field_copy(def: &NodeDef) -> Vec<String> {
    let mut copy = Vec::new();
    for key in def.params.keys() {
        let field = def.params.field(key).unwrap();
        if field["x-hidden"] == true {
            continue;
        }
        for name in ["description", "x-help", "x-zero-label", "x-presets-label"] {
            copy.extend(field[name].as_str().map(str::to_owned));
        }
        if let Some(labels) = field["x-labels"].as_object() {
            copy.extend(labels.values().filter_map(Value::as_str).map(str::to_owned));
        }
        if let Some(presets) = field["x-presets"].as_array() {
            copy.extend(
                presets
                    .iter()
                    .filter_map(|p| p["label"].as_str())
                    .map(str::to_owned),
            );
        }
    }
    copy
}

#[test]
fn gives_every_visible_node_a_description_and_no_internal_names() {
    let banned = Regex::new(r"(?i)playwright|remotion|chromium|ffmpeg|ffprobe|lottie|after effects|page\.evaluate|safeStorage|\.mp4|\.json|crf|dpr\b").unwrap();
    let undescribed: Vec<String> = registry()
        .visible()
        .filter(|d| d.description.is_none())
        .map(|d| d.kind.to_string())
        .collect();
    assert!(undescribed.is_empty(), "{undescribed:?}");
    let named: Vec<String> = registry()
        .visible()
        .filter(|d| {
            let mut text = vec![d.title.clone(), d.description.clone().unwrap_or_default()];
            text.extend(field_copy(d));
            banned.is_match(&text.join(" "))
        })
        .map(|d| d.kind.to_string())
        .collect();
    assert!(named.is_empty(), "{named:?}");
}

const LONG_TEXT: &str = "A long piece of text that runs well past the width of a card";

fn candidates(field: &Value, current: Option<&Value>) -> Vec<Value> {
    let options = |f: &Value| {
        f["enum"].as_array().cloned().or_else(|| {
            f["oneOf"]
                .as_array()
                .map(|o| o.iter().filter_map(|x| x.get("const").cloned()).collect())
        })
    };
    match field["type"].as_str() {
        Some("boolean") => vec![json!(false), json!(true)],
        Some("number" | "integer") => vec![json!(0), json!(1), json!(5), json!(500)],
        Some("string") if options(field).is_none() => {
            vec![json!(""), json!("Save"), json!("de-DE"), json!(LONG_TEXT)]
        }
        Some("array") if options(&field["items"]).is_some() => {
            vec![json!([]), Value::Array(options(&field["items"]).unwrap())]
        }
        _ => options(field).unwrap_or_else(|| current.cloned().into_iter().collect()),
    }
}

fn longest_card(def: &NodeDef) -> usize {
    if !def.has_card() {
        return 0;
    }
    let ports = all_ports(def);
    let frames = |n: u32| ports.iter().map(|p| (p.clone(), n)).collect();
    let items = |n: usize| ports.iter().map(|p| (p.clone(), n)).collect();
    let log = make_log(
        vec![
            action(json!({ "id": "a", "kind": "click", "t": 1000 })),
            action(json!({ "id": "b", "kind": "type", "t": 2000 })),
        ],
        14_500.0,
    );
    let result = if def.kind == "capture.record" {
        record_result(&log)
    } else {
        node_result(json!({}))
    };
    let contexts = [
        (CardInputs::default(), None),
        (
            CardInputs {
                connected: ports.clone(),
                clip_frames: Some(90),
                frames: frames(60),
                items: items(3),
                secrets: Some(vec!["work".into()]),
                ..CardInputs::default()
            },
            Some(&result),
        ),
        (
            CardInputs {
                connected: ports.clone(),
                clip_frames: Some(12),
                frames: frames(8),
                items: items(1),
                ..CardInputs::default()
            },
            None,
        ),
    ];
    let mut longest = 0;
    for (inputs, result) in &contexts {
        let count = |params: &Map<String, Value>| {
            def.parse(params, None)
                .map_or(0, |v| rows_for(def, &v, inputs, *result).len())
        };
        let mut params = def.defaults();
        let mut best = count(&params);
        let mut improved = true;
        while improved {
            improved = false;
            for key in def.params.keys().to_vec() {
                for value in candidates(def.params.field(&key).unwrap(), params.get(&key)) {
                    let mut next = params.clone();
                    next.insert(key.clone(), value);
                    let rows = count(&next);
                    if rows > best {
                        (params, best, improved) = (next, rows, true);
                    }
                }
            }
        }
        longest = longest.max(best);
    }
    longest
}

#[test]
fn never_builds_more_rows_than_a_card_shows_whatever_its_settings() {
    let over: Vec<String> = registry()
        .all()
        .iter()
        .filter_map(|d| {
            let longest = longest_card(d);
            (longest > MAX_CARD_ROWS).then(|| format!("{}: {longest}", d.kind))
        })
        .collect();
    assert!(over.is_empty(), "{over:?}");
}
