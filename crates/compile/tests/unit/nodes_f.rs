use indexmap::IndexMap;
use opsis_compile::compile::compile_workflow;
use opsis_compile::node::{CardInputs, Category, InputMedia};
use opsis_compile::nodes::grade::{
    grade_changes_nothing, grade_needs_video, is_neutral_entry, nodes,
};
use opsis_compile::ports::can_connect;
use opsis_compile::registry::node_def;
use opsis_types::cards::Tone;
use opsis_types::values::{CompileValue, MediaKind, StyleEntry};
use opsis_types::workflow::PortType;
use serde_json::{Value, json};

use crate::helpers::{Graph, card_texts, compile_node, node_card, parse_params};

fn entry(kind: &str, params: Value) -> StyleEntry {
    StyleEntry {
        node_id: kind.into(),
        kind: kind.into(),
        params: parse_params(kind, params).unwrap(),
    }
}

fn grade_entries(out: &opsis_compile::compile::CompileResult, id: &str) -> Vec<StyleEntry> {
    match &out.values[id]["grade"] {
        CompileValue::Grade(g) => g.entries.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn makes_grade_its_own_port_type_that_only_grade_nodes_make() {
    assert!(can_connect(PortType::Grade, PortType::Grade));
    for source in [
        PortType::Clip,
        PortType::Media,
        PortType::RecordingStyle,
        PortType::Recording,
    ] {
        assert!(!can_connect(source, PortType::Grade), "{source:?}");
    }
    assert!(
        !can_connect(PortType::Grade, PortType::RecordingStyle)
            && !can_connect(PortType::Grade, PortType::Media)
    );
    for def in nodes() {
        assert_eq!(def.category, Category::Grade, "{}", def.kind);
        assert_eq!(
            def.inputs.iter().map(|(_, p)| p.kind).collect::<Vec<_>>(),
            [PortType::Grade]
        );
        assert_eq!(
            def.outputs.iter().map(|(_, p)| p.kind).collect::<Vec<_>>(),
            [PortType::Grade]
        );
        assert!(def.input("grade").unwrap().optional);
    }
}

#[test]
fn builds_the_grade_list_in_chain_order_passing_skipped_nodes_through() {
    let mut g = Graph::default();
    g.node("a", "grade.colour", json!({ "contrast": 1.2 }));
    g.node("b", "grade.mono", json!({}));
    g.node("c", "grade.texture", json!({ "grain": 0.4 }));
    g.link_ports("a", "b", "grade", "grade")
        .link_ports("b", "c", "grade", "grade");
    let mut wf = g.build();
    let entries = grade_entries(&compile_workflow(&wf, None), "c");
    assert_eq!(
        entries
            .iter()
            .map(|e| (e.node_id.as_str(), e.kind.as_str()))
            .collect::<Vec<_>>(),
        [
            ("a", "grade.colour"),
            ("b", "grade.mono"),
            ("c", "grade.texture")
        ]
    );
    assert_eq!(entries[0].params["contrast"], 1.2);
    wf.nodes.iter_mut().find(|n| n.id == "b").unwrap().disabled = Some(true);
    assert_eq!(
        grade_entries(&compile_workflow(&wf, None), "c")
            .iter()
            .map(|e| e.node_id.as_str())
            .collect::<Vec<_>>(),
        ["a", "c"]
    );
    match &compile_node("grade.autoFix", json!({}), json!({}))["grade"] {
        CompileValue::Grade(g) => assert_eq!(
            g.entries
                .iter()
                .map(|e| e.kind.as_str())
                .collect::<Vec<_>>(),
            ["grade.autoFix"]
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn knows_which_steps_change_nothing_and_which_need_a_video() {
    let neutral: Vec<String> = nodes()
        .iter()
        .filter(|d| is_neutral_entry(&entry(&d.kind, json!({}))))
        .map(|d| d.kind.to_string())
        .collect();
    assert_eq!(
        neutral,
        [
            "grade.colour",
            "grade.texture",
            "grade.lens",
            "grade.autoFix"
        ]
    );
    assert!(grade_changes_nothing(&[]));
    assert!(grade_changes_nothing(&[
        entry("grade.colour", json!({})),
        entry("grade.lens", json!({}))
    ]));
    assert!(!grade_changes_nothing(&[
        entry("grade.colour", json!({})),
        entry("grade.mono", json!({}))
    ]));
    assert!(grade_changes_nothing(&[entry(
        "grade.mono",
        json!({ "strength": 0 })
    )]));
    assert!(!is_neutral_entry(&entry(
        "grade.colour",
        json!({ "shadowsAmount": 0.5 })
    )));
    assert!(is_neutral_entry(&entry(
        "grade.colour",
        json!({ "shadows": "", "shadowsAmount": 0.5 })
    )));
    assert!(grade_needs_video(&[entry(
        "grade.lens",
        json!({ "trails": 0.4 })
    )]));
    assert!(grade_needs_video(&[entry(
        "grade.texture",
        json!({ "grain": 0.4 })
    )]));
    assert!(!grade_needs_video(&[
        entry("grade.texture", json!({ "vignette": 0.4 })),
        entry("grade.colour", json!({}))
    ]));
}

fn texts(kind: &str, params: Value) -> Vec<String> {
    card_texts(kind, params, CardInputs::default())
}

#[test]
fn shows_what_each_grade_step_changes_on_its_card() {
    let none = node_card("grade.colour", json!({}), CardInputs::default())[0].clone();
    assert_eq!(
        (none.text.as_str(), none.tone),
        ("No change", Some(Tone::Muted))
    );
    assert_eq!(
        texts(
            "grade.colour",
            json!({ "contrast": 1.3, "warmth": 0.5, "curve": "vintage" })
        ),
        ["3 changes · Contrast, Warmth, Tone curve", "Vintage"]
    );
    assert_eq!(
        texts("grade.texture", json!({ "grain": 0.3 })),
        ["1 change · Film grain"]
    );
    assert_eq!(
        texts("grade.lens", json!({ "trails": 0.3, "fringe": 0.2 })),
        ["2 changes · Colour fringing, Motion trails"]
    );
    assert_eq!(
        texts("grade.autoFix", json!({ "deband": true })),
        ["1 change · Remove banding"]
    );
    let ask = node_card("grade.lookFile", json!({}), CardInputs::default())[0].clone();
    assert_eq!(
        (ask.text.as_str(), ask.tone),
        ("Choose a look file", Some(Tone::Warn))
    );
    assert_eq!(
        texts(
            "grade.lookFile",
            json!({ "file": "looks/warm.cube", "strength": 0.5 })
        ),
        ["warm.cube", "50% strength"]
    );
    let file = "assets/looks/warm.cube";
    let gone = node_card(
        "grade.lookFile",
        json!({ "file": file }),
        CardInputs {
            missing_files: vec![file.into()],
            ..CardInputs::default()
        },
    )[0]
    .clone();
    assert_eq!(
        (gone.text.as_str(), gone.tone),
        ("warm.cube isn't in the project", Some(Tone::Warn))
    );
    assert_eq!(
        node_card(
            "grade.lookFile",
            json!({ "file": file }),
            CardInputs::default()
        )[0]
        .tone,
        None
    );
    assert_eq!(texts("grade.mono", json!({})), ["Black and white"]);
    let tint = node_card(
        "grade.mono",
        json!({ "kind": "tint", "tint": "#ff0000" }),
        CardInputs::default(),
    )[0]
    .clone();
    assert_eq!(
        (tint.text.as_str(), tint.swatches),
        ("Tint", Some(vec!["#ff0000".to_owned()]))
    );
    assert_eq!(
        node_card(
            "grade.mono",
            json!({ "kind": "tint", "tint": "" }),
            CardInputs::default()
        )[0]
        .swatches,
        Some(vec!["#d8a15b".to_owned()])
    );
    let presets = node_def("grade.colour")
        .unwrap()
        .params
        .field("brightness")
        .unwrap()["x-presets"]
        .clone();
    let labels: Vec<&str> = presets
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["label"].as_str().unwrap())
        .collect();
    assert_eq!(
        labels,
        [
            "Natural",
            "Warm",
            "Cool",
            "Punchy",
            "Soft",
            "Black and white",
            "Old film"
        ]
    );
    assert!(presets.as_array().unwrap().iter().all(|p| {
        p["values"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(",")
            == "brightness,contrast,saturation,warmth"
    }));
}

fn graded(steps: &[(&str, Value)], kind: Option<MediaKind>) -> CardInputs {
    CardInputs {
        grade: IndexMap::from([(
            "grade".to_owned(),
            steps.iter().map(|(k, p)| entry(k, p.clone())).collect(),
        )]),
        media: kind
            .map(|k| {
                (
                    "media".to_owned(),
                    InputMedia {
                        kind: k,
                        has_audio: None,
                    },
                )
            })
            .into_iter()
            .collect(),
        ..CardInputs::default()
    }
}

#[test]
fn sums_up_the_grade_an_apply_grade_node_applies() {
    let ask = node_card("media.applyGrade", json!({}), CardInputs::default())[0].clone();
    assert_eq!(
        (ask.text.as_str(), ask.tone),
        ("Connect a grade", Some(Tone::Warn))
    );
    let steps = [
        ("grade.colour", json!({ "contrast": 1.2 })),
        ("grade.colour", json!({ "warmth": 0.2 })),
        ("grade.texture", json!({ "grain": 0.2 })),
    ];
    assert_eq!(
        card_texts("media.applyGrade", json!({}), graded(&steps, None)),
        ["3 steps: Colour and light, Texture", "MP4 · High quality"]
    );
    assert!(
        card_texts(
            "media.applyGrade",
            json!({}),
            graded(
                &[("grade.colour", json!({})), ("grade.lens", json!({}))],
                None
            )
        )
        .contains(&"The grade changes nothing".to_owned())
    );
    assert_eq!(
        texts("media.applyGrade", json!({ "codec": "h265" })).join(","),
        "Connect a grade,MP4 (HEVC) · High quality · may not play in a film"
    );
    assert_eq!(
        texts("media.applyGrade", json!({ "codec": "prores" })).join(","),
        "Connect a grade,ProRes · can't play in a film"
    );
    assert_eq!(
        card_texts(
            "media.applyGrade",
            json!({}),
            graded(&[("grade.mono", json!({}))], Some(MediaKind::Image))
        )[1],
        "PNG"
    );
    let trails = [("grade.lens", json!({ "trails": 0.5 }))];
    let image = node_card(
        "media.applyGrade",
        json!({}),
        graded(&trails, Some(MediaKind::Image)),
    );
    let last = image.last().unwrap();
    assert_eq!(
        (last.text.as_str(), last.tone),
        ("Trails and film grain need a video", Some(Tone::Warn))
    );
    assert!(
        !card_texts(
            "media.applyGrade",
            json!({}),
            graded(&trails, Some(MediaKind::Video))
        )
        .contains(&"Trails and film grain need a video".to_owned())
    );
}
