use opsis_compile::catalog::{catalog_entry, node_catalog};
use opsis_compile::node::{NodeDef, Platform, PortSpec};
use opsis_compile::registry::registry;
use opsis_compile::size::estimate_card_height;
use serde_json::{Map, Value, json};

fn ports(list: &[(&'static str, PortSpec)]) -> Value {
    Value::Array(
        list.iter()
            .map(|(id, p)| json!({ "id": id, "type": p.kind.id(), "label": p.label.as_deref(), "optional": p.optional, "many": p.many }))
            .collect(),
    )
}

fn host_defaults(def: &NodeDef) -> Option<Value> {
    def.host_defaults.map(|f| {
        let by_platform: Map<String, Value> = [
            ("linux", Platform::Linux),
            ("darwin", Platform::Mac),
            ("win32", Platform::Windows),
        ]
        .into_iter()
        .map(|(name, p)| (name.to_owned(), Value::Object(f(p))))
        .collect();
        Value::Object(by_platform)
    })
}

fn node_metadata(def: &NodeDef) -> Value {
    let defaults = def.defaults();
    json!({
        "type": def.kind,
        "title": def.title,
        "category": def.category,
        "description": def.description,
        "phase": def.phase,
        "inputs": ports(&def.inputs),
        "outputs": ports(&def.outputs),
        "virtual": def.is_virtual,
        "action": def.action,
        "executes": def.executes,
        "usesProjectSize": def.uses_project_size,
        "element": def.element,
        "hidden": def.hidden,
        "cardHeight": def.card_height,
        "estimatedHeight": estimate_card_height(Some(def)),
        "compiles": def.has_compile(),
        "hasCard": def.has_card(),
        "mutates": def.parse(&defaults, None).ok().and_then(|v| def.mutation(&v)),
        "hostDefaults": host_defaults(def),
        "defaults": defaults,
    })
}

#[test]
fn node_definitions_are_as_recorded() {
    let nodes: Vec<Value> = registry().all().iter().map(node_metadata).collect();
    opsis_golden::check("compile/nodes.json", &Value::Array(nodes));
}

#[test]
fn the_catalogue_is_as_recorded() {
    let entries: Vec<Value> = registry()
        .all()
        .iter()
        .filter_map(|def| catalog_entry(&def.kind))
        .map(|entry| serde_json::to_value(entry).expect("entry serializes"))
        .collect();
    opsis_golden::check(
        "compile/catalog.json",
        &json!({ "list": node_catalog(), "entries": entries }),
    );
}
