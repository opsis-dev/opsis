use std::collections::BTreeMap;

use opsis_compile::node::NodeDef;
use opsis_compile::registry::registry;
use opsis_types::workflow::PortType;

const SAME_STEM_ALLOWED: &[(&str, &str, &str)] = &[
    (
        "media.speed",
        "media.pitch",
        "speed changes the length, pitch only the voice",
    ),
    (
        "fx.code-reveal",
        "fx.code-diff-wipe",
        "one shows a file, the other the change between two",
    ),
    (
        "fx.file-tree-reveal",
        "fx.drag-drop-flow",
        "a folder tree and a file dropped into a page are different scenes",
    ),
    (
        "media.fixClicks",
        "media.fixClipping",
        "each fix is its own node so a file runs only the fixes it needs",
    ),
    (
        "media.hum",
        "media.removeNoise",
        "each fix is its own node so a file runs only the fixes it needs",
    ),
    (
        "fx.logo-set",
        "fx.logo-reveal",
        "a set shows many logos, a reveal brings in one",
    ),
    (
        "fx.text-highlight",
        "fx.text-mask-video",
        "one marks words in a line, the other fills letters with a picture",
    ),
    (
        "browser.navigate",
        "browser.goOffline",
        "one opens an address, the other cuts the page off from the network",
    ),
];

fn port_types(def: &NodeDef) -> (Vec<PortType>, Vec<PortType>) {
    let mut inputs: Vec<PortType> = def.inputs.iter().map(|(_, spec)| spec.kind).collect();
    inputs.extend(
        def.family
            .as_ref()
            .map(opsis_compile::node::PortFamily::kind),
    );
    let mut outputs: Vec<PortType> = def.outputs.iter().map(|(_, spec)| spec.kind).collect();
    for list in [&mut inputs, &mut outputs] {
        list.sort_by_key(|kind| format!("{kind:?}"));
        list.dedup();
    }
    (inputs, outputs)
}

fn stem(title: &str) -> String {
    title
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_lowercase()
}

#[test]
fn no_two_nodes_do_the_same_thing() {
    let mut seen: BTreeMap<String, Vec<&NodeDef>> = BTreeMap::new();
    for def in registry().visible() {
        let (inputs, outputs) = port_types(def);
        let key = format!("{} {inputs:?} {outputs:?}", stem(&def.title));
        seen.entry(key).or_default().push(def);
    }
    let allowed = |a: &str, b: &str| {
        SAME_STEM_ALLOWED
            .iter()
            .any(|(x, y, _)| (*x == a && *y == b) || (*x == b && *y == a))
    };
    let mut clashes = Vec::new();
    for defs in seen.values() {
        for (i, a) in defs.iter().enumerate() {
            for b in &defs[i + 1..] {
                if !allowed(&a.kind, &b.kind) {
                    clashes.push(format!(
                        "{} ({}) and {} ({})",
                        a.title, a.kind, b.title, b.kind
                    ));
                }
            }
        }
    }
    assert!(
        clashes.is_empty(),
        "these nodes take and give the same ports under one title: {clashes:#?}"
    );
}

fn draws(value: &serde_json::Value, component: &str) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            map.get("component").and_then(serde_json::Value::as_str) == Some(component)
                || map.values().any(|v| draws(v, component))
        }
        serde_json::Value::Array(items) => items.iter().any(|v| draws(v, component)),
        _ => false,
    }
}

fn background() -> serde_json::Value {
    let clip = crate::helpers::compile_clip(
        "fx.background",
        serde_json::json!({}),
        serde_json::json!({}),
    );
    serde_json::to_value(opsis_types::values::CompileValue::Clip(clip)).unwrap()
}

#[test]
fn every_group_compiles_at_every_kind() {
    use opsis_compile::nodes::groups::{KIND, groups};
    let clip = background();
    for group in groups() {
        for kind in &group.kinds {
            let clip = crate::helpers::compile_clip(
                group.node_type,
                serde_json::json!({ KIND: kind.id }),
                serde_json::json!({ "clip": clip }),
            );
            assert!(
                draws(&crate::helpers::clip_value(&clip), kind.element),
                "{} at {} doesn't draw {}",
                group.node_type,
                kind.id,
                kind.element
            );
        }
    }
}

#[test]
fn a_group_at_its_defaults_draws_each_kind_as_its_element_does() {
    use opsis_compile::nodes::elements::compile_element;
    use opsis_compile::nodes::groups::{KIND, groups};
    use opsis_types::values::CompileValue;
    let fed = background();
    let input = match serde_json::from_value::<CompileValue>(fed.clone()).unwrap() {
        CompileValue::Clip(clip) => clip,
        other => panic!("{other:?}"),
    };
    let settings = opsis_types::workflow::WorkflowSettings::default();
    for group in groups() {
        for kind in &group.kinds {
            let grouped = crate::helpers::compile_clip(
                group.node_type,
                serde_json::json!({ KIND: kind.id }),
                serde_json::json!({ "clip": fed }),
            );
            let mut own = kind.meta().default_props.clone();
            own.extend(kind.fixed.clone());
            let alone = compile_element(kind.meta(), "n", &own, Some(&input), &settings);
            assert_eq!(
                crate::helpers::clip_value(&grouped),
                crate::helpers::clip_value(&alone),
                "{} at {} differs from {} on its own",
                group.node_type,
                kind.id,
                kind.element
            );
        }
    }
}

const DROPS_THE_TRANSITION: &[(&str, &str)] = &[(
    "timeline.stack",
    "layers stack clips, so a transition on a layer has nothing to join",
)];

#[test]
fn every_clip_node_keeps_a_marked_clips_transition() {
    use opsis_types::values::CompileValue;
    let marked = crate::helpers::compile_clip(
        "timeline.transition",
        serde_json::json!({ "kind": "fade" }),
        serde_json::json!({ "clip": background() }),
    );
    let enter = marked.enter.clone();
    assert!(enter.is_some());
    let fed = serde_json::to_value(CompileValue::Clip(marked)).unwrap();
    let mut checked = 0;
    for def in registry().visible() {
        let gives_clip = def
            .outputs
            .iter()
            .any(|(name, spec)| *name == "clip" && spec.kind == PortType::Clip);
        let mut ports: Vec<String> = def
            .inputs
            .iter()
            .filter(|(_, spec)| spec.kind == PortType::Clip)
            .map(|(name, _)| (*name).to_owned())
            .collect();
        ports.extend(
            def.family
                .as_ref()
                .filter(|f| f.kind() == PortType::Clip)
                .map(|f| f.port(0)),
        );
        if !gives_clip
            || ports.is_empty()
            || DROPS_THE_TRANSITION
                .iter()
                .any(|(kind, _)| *kind == def.kind)
        {
            continue;
        }
        let inputs: serde_json::Map<String, serde_json::Value> =
            ports.into_iter().map(|port| (port, fed.clone())).collect();
        let out = crate::helpers::compile_node(&def.kind, serde_json::json!({}), inputs.into());
        let Some(CompileValue::Clip(clip)) = out.get("clip") else {
            panic!("{} gives no clip", def.kind);
        };
        assert_eq!(
            clip.enter, enter,
            "{} drops the clip's transition",
            def.kind
        );
        checked += 1;
    }
    assert!(checked > 10, "only {checked} clip nodes were checked");
}
