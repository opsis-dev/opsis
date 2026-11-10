use opsis_compile::edits::timeline::{TimelineEdits, apply_removal};
use opsis_compile::graph::validate_workflow;
use opsis_compile::ids::SaltedIds;
use opsis_compile::registry::node_def;
use opsis_types::edits::Removal;
use opsis_types::timeline::TimelineItem;
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};

use crate::helpers::{Graph, TITLE, export_film, film, flat_film};

fn shown(kind: &str, connected: &[&str]) -> Vec<(String, String, bool)> {
    node_def(kind)
        .unwrap()
        .input_ports(connected.iter().copied())
        .into_iter()
        .map(|(port, label, spec)| (port, label, spec.optional))
        .collect()
}

fn names(ports: &[(String, String, bool)]) -> Vec<&str> {
    ports.iter().map(|(p, _, _)| p.as_str()).collect()
}

fn labels(ports: &[(String, String, bool)]) -> Vec<&str> {
    ports.iter().map(|(_, l, _)| l.as_str()).collect()
}

fn clips(count: usize, kind: &str, params: Value) -> Workflow {
    film(|g| {
        for i in 0..count {
            g.node(
                &format!("c{i}"),
                TITLE,
                json!({ "clipDuration": 60 + i * 5 }),
            );
            g.link_ports(&format!("c{i}"), "box", "clip", &format!("in{}", i + 1));
        }
        g.node("box", kind, params);
        "box".into()
    })
}

fn ports_of(wf: &Workflow, node: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = wf
        .edges
        .iter()
        .filter(|e| e.target == node)
        .map(|e| (e.target_handle.clone(), e.source.clone()))
        .collect();
    found.sort_by_key(|(port, _)| port[2..].parse::<usize>().unwrap_or(0));
    found
}

#[test]
fn shows_one_empty_input_after_the_last_connected_one() {
    assert_eq!(
        names(&shown("timeline.sequence", &[])),
        ["transition", "in1", "in2"]
    );
    assert_eq!(
        names(&shown("timeline.sequence", &["in1", "in2", "in3"])),
        ["transition", "in1", "in2", "in3", "in4"]
    );
    assert_eq!(
        names(&shown("timeline.sequence", &["in1", "in9"])).len(),
        11
    );
    assert_eq!(
        labels(&shown("timeline.stack", &["in1", "in2", "in3"])),
        ["Layer 1 (back)", "Layer 2", "Layer 3 (front)", "Layer 4"]
    );
    let join = shown("media.concat", &["in1", "in2", "in3"]);
    assert_eq!(names(&join), ["in1", "in2", "in3", "in4"]);
    assert_eq!(
        join.iter()
            .map(|(_, _, optional)| *optional)
            .collect::<Vec<_>>(),
        [false, false, true, true]
    );
    let def = node_def("timeline.sequence").unwrap();
    assert!(def.input("in42").is_some());
    for odd in ["in0", "in", "in01", "in2a", "clip", "cuts"] {
        assert!(def.input(odd).is_none(), "{odd}");
    }
}

#[test]
fn asks_for_the_files_join_files_needs_and_takes_any_number_more() {
    let mut g = Graph::default();
    g.node("a", "media.import", json!({ "path": "assets/a.mp4" }));
    g.node("join", "media.concat", json!({}));
    g.link_ports("a", "join", "media", "in1");
    let issues: Vec<String> = validate_workflow(&g.build(), None)
        .into_iter()
        .filter(|i| i.node_id.as_deref() == Some("join"))
        .map(|i| i.message)
        .collect();
    assert_eq!(issues, ["Connect something to “File 2”"]);
    for i in 2..=7 {
        g.node(
            &format!("f{i}"),
            "media.import",
            json!({ "path": "assets/b.mp4" }),
        );
        g.link_ports(&format!("f{i}"), "join", "media", &format!("in{i}"));
    }
    assert!(
        validate_workflow(&g.build(), None)
            .iter()
            .all(|i| i.node_id.as_deref() != Some("join"))
    );
}

#[test]
fn plays_a_clips_own_transition_past_the_sixth_clip() {
    let wf = film(|g| {
        for i in 0..9 {
            let id = format!("c{i}");
            g.node(&id, TITLE, json!({ "clipDuration": 60 + i * 5 }));
            let port = format!("in{}", i + 1);
            if i == 7 {
                g.node(
                    "marked",
                    "timeline.transition",
                    json!({ "kind": "wipe", "durationInFrames": 20 }),
                );
                g.link(&id, "marked")
                    .link_ports("marked", "box", "clip", &port);
            } else {
                g.link_ports(&id, "box", "clip", &port);
            }
        }
        g.node(
            "shared",
            "timeline.transition",
            json!({ "kind": "fade", "durationInFrames": 10 }),
        );
        g.node("box", "timeline.sequence", json!({}));
        g.link_ports("shared", "box", "transition", "transition");
        "box".into()
    });
    let items: Vec<TimelineItem> = export_film(&wf, None, "out").tracks[0].items.clone();
    assert_eq!(items.len(), 9);
    let entered: Vec<Option<String>> = items
        .iter()
        .map(|i| i.base().enter.as_ref().map(|e| e.kind.clone()))
        .collect();
    assert_eq!(entered[7].as_deref(), Some("wipe"));
    assert_eq!(entered[8].as_deref(), Some("fade"));
}

#[test]
fn splits_a_clip_inside_a_sequence_of_seven_without_a_new_sequence() {
    let wf = clips(7, "timeline.sequence", json!({}));
    let flat = flat_film(&wf);
    let last = flat
        .bars
        .iter()
        .find(|b| b.node_id == "c6")
        .expect("the seventh clip");
    let mut ids = SaltedIds::new("t");
    let mut edits = TimelineEdits {
        workflow: wf.clone(),
        fps: wf.settings.fps,
        source: Some("out".to_owned()),
        ids: &mut ids,
    };
    edits
        .split_bar(last, f64::midpoint(last.from, last.to))
        .expect("the split");
    let split = edits.workflow;
    assert_eq!(split.nodes.len(), wf.nodes.len() + 2);
    assert_eq!(
        split
            .nodes
            .iter()
            .filter(|n| n.kind == "timeline.sequence")
            .count(),
        1
    );
    let ports: Vec<String> = ports_of(&split, "box")
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    assert_eq!(ports, (1..=8).map(|i| format!("in{i}")).collect::<Vec<_>>());
    assert!(split.node("box").unwrap().params.is_empty());
}

#[test]
fn closes_the_gap_when_a_clip_is_removed() {
    let wf = clips(5, "timeline.sequence", json!({}));
    let mut removed = wf;
    apply_removal(
        &Removal {
            node_ids: vec!["c1".to_owned()],
            edge_ids: Vec::new(),
        },
        &mut removed,
    );
    assert_eq!(
        ports_of(&removed, "box"),
        [("in1", "c0"), ("in2", "c2"), ("in3", "c3"), ("in4", "c4")]
            .map(|(p, s)| (p.to_owned(), s.to_owned()))
    );
}
