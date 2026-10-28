use opsis_compile::node::NodeDef;
use opsis_compile::registry::node_def;

pub fn known_types<'a>(file: &str, types: impl IntoIterator<Item = &'a str>) {
    let mut unknown: Vec<&str> = types
        .into_iter()
        .filter(|kind| node_def(kind).is_none())
        .collect();
    unknown.sort_unstable();
    unknown.dedup();
    assert!(
        unknown.is_empty(),
        "{file} names node types that don't exist: {unknown:?}. Rename them in the file; recording without them would drop what was recorded for them."
    );
}

pub fn def_of(kind: &str) -> &'static NodeDef {
    node_def(kind).expect("a node type checked by known_types")
}
