use opsis_compile::clip::clip_to_spec;
use opsis_compile::compile::compile_workflow;
use opsis_compile::elements::overlay::captions::CAPTIONS_ELEMENT;
use opsis_compile::elements::{ElementKind, ElementMeta, element_node_type};
use opsis_compile::hosted::hosted;
use opsis_compile::node::CLIP_PORT;
use opsis_compile::nodes::device_frame::DEVICE_FRAME_TYPE;
use opsis_compile::nodes::groups::{KIND, groups};
use opsis_compile::nodes::media::{CLIP_ELEMENTS, USE_AS_CLIP_TYPE};
use opsis_compile::nodes::text::{CAPTIONS_FROM_SCRIPT_TYPE, CAPTIONS_PORT, SHOW_CAPTIONS_TYPE};
use opsis_compile::nodes::timeline::SEQUENCE_TYPE;
use opsis_compile::nodes::transitions::{TRANSITION_PORT, TRANSITION_TYPE};
use opsis_types::numbers::whole;
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::CompileValue;
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Value, json};

const SOLID_FRAMES: u32 = 90;
const ELEMENT_NODE: &str = "element";

pub const ORACLE_POSITIONS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

fn node(id: &str, kind: &str, params: Value) -> Value {
    let mut node = json!({ "id": id, "type": kind, "position": { "x": 0, "y": 0 } });
    node["params"] = params;
    node
}

fn edge(source: &str, target: &str, target_handle: &str) -> Value {
    port_edge(source, CLIP_PORT, target, target_handle)
}

fn port_edge(source: &str, source_handle: &str, target: &str, target_handle: &str) -> Value {
    json!({ "id": format!("{source}->{target}.{target_handle}"), "source": source, "sourceHandle": source_handle, "target": target, "targetHandle": target_handle })
}

fn solid(id: &str, color: &str) -> Value {
    node(
        id,
        "fx.background",
        json!({ "solid": { "color": color }, "clipDuration": SOLID_FRAMES }),
    )
}

fn device_angle(key: &str) -> Option<Value> {
    Some(match key {
        "laptop-frame" => json!({ "angle": "flat", "device": "laptop" }),
        "phone-frame" => json!({ "angle": "flat", "device": "phone" }),
        "screen-3d" => json!({ "angle": "three-d" }),
        "device-mockup-zoom" => json!({ "angle": "pull-back" }),
        "multi-device-lineup" => json!({ "angle": "lineup" }),
        _ => return None,
    })
}

type Feeds = (Vec<Value>, Vec<Value>);

fn joined(transition: &str) -> Feeds {
    (
        vec![
            solid("a", "#334155"),
            solid("b", "#7c3aed"),
            node("join", TRANSITION_TYPE, json!({ "kind": transition })),
        ],
        vec![
            edge("a", ELEMENT_NODE, "in1"),
            edge("b", ELEMENT_NODE, "in2"),
            port_edge("join", TRANSITION_PORT, ELEMENT_NODE, TRANSITION_PORT),
        ],
    )
}

fn wrapped() -> Feeds {
    (
        vec![solid("in", "#334155")],
        vec![edge("in", ELEMENT_NODE, CLIP_PORT)],
    )
}

fn fed(meta: &ElementMeta) -> Feeds {
    if meta.kind == ElementKind::Wrapper {
        wrapped()
    } else {
        (Vec::new(), Vec::new())
    }
}

fn node_for(meta: &ElementMeta) -> (String, Value, Feeds) {
    if meta.key == CAPTIONS_ELEMENT {
        let script = node("script", CAPTIONS_FROM_SCRIPT_TYPE, json!({}));
        let link = port_edge("script", CAPTIONS_PORT, ELEMENT_NODE, CAPTIONS_PORT);
        return (
            SHOW_CAPTIONS_TYPE.to_owned(),
            json!({}),
            (vec![script], vec![link]),
        );
    }
    if matches!(
        meta.kind,
        ElementKind::Presentation | ElementKind::Sequencer
    ) {
        return (SEQUENCE_TYPE.to_owned(), json!({}), joined(&meta.key));
    }
    if let Some(params) = device_angle(&meta.key) {
        return (DEVICE_FRAME_TYPE.to_owned(), params, wrapped());
    }
    if CLIP_ELEMENTS.contains(&meta.key.as_str()) {
        return (
            USE_AS_CLIP_TYPE.to_owned(),
            json!({}),
            (Vec::new(), Vec::new()),
        );
    }
    let grouped = groups().iter().find_map(|g| {
        g.kinds
            .iter()
            .find(|k| k.element == meta.key)
            .map(|k| (g.node_type.to_owned(), json!({ KIND: k.id })))
    });
    let (kind, params) = grouped.unwrap_or_else(|| (element_node_type(&meta.key), json!({})));
    (kind, params, fed(meta))
}

pub fn workflow_for(meta: &ElementMeta) -> Workflow {
    let (kind, params, (inputs, links)) = node_for(meta);
    let mut nodes = inputs;
    nodes.push(node(ELEMENT_NODE, &kind, params));
    serde_json::from_value(json!({ "version": 1, "id": "elements", "name": "Elements", "nodes": nodes, "edges": links, "markers": [] })).expect("a workflow")
}

#[derive(Debug, thiserror::Error)]
pub enum ElementError {
    #[error("{0}")]
    Compile(String),
    #[error("no clip")]
    NoClip,
}

pub fn element_spec(meta: &ElementMeta, project_base: &str) -> Result<TimelineSpec, ElementError> {
    let settings = WorkflowSettings::default();
    let compiled = compile_workflow(&workflow_for(meta), None);
    if let Some(error) = compiled.errors.get(ELEMENT_NODE) {
        return Err(ElementError::Compile(error.clone()));
    }
    let Some(CompileValue::Clip(clip)) = compiled
        .values
        .get(ELEMENT_NODE)
        .and_then(|v| v.get(CLIP_PORT))
    else {
        return Err(ElementError::NoClip);
    };
    Ok(hosted(&TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some(project_base.to_owned()),
        }),
        ..clip_to_spec(clip, &settings)
    }))
}

pub fn frame_at(duration_in_frames: u32, at: f64) -> u32 {
    let last = duration_in_frames.saturating_sub(1);
    whole((f64::from(last) * at.clamp(0.0, 1.0)).floor())
}

pub fn frames_at(duration_in_frames: u32, positions: &[f64]) -> Vec<u32> {
    let mut frames: Vec<u32> = positions
        .iter()
        .map(|&at| frame_at(duration_in_frames, at))
        .collect();
    frames.sort_unstable();
    frames.dedup();
    frames
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn frames_are_ascending_distinct_and_inside_the_film(duration in 1u32..10_000, positions in proptest::collection::vec(-1.0f64..2.0, 0..8)) {
            let frames = frames_at(duration, &positions);
            prop_assert!(frames.windows(2).all(|w| w[0] < w[1]));
            prop_assert!(frames.iter().all(|&f| f < duration));
        }

        #[test]
        fn the_ends_are_the_first_and_last_frames(duration in 1u32..10_000) {
            prop_assert_eq!(frame_at(duration, 0.0), 0);
            prop_assert_eq!(frame_at(duration, 1.0), duration - 1);
        }
    }
}
