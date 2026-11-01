use std::collections::HashMap;
use std::sync::LazyLock;

use opsis_types::workflow::WorkflowNode;

use crate::node::{Category, NodeDef};
use crate::nodes;
use crate::params::Values;

#[derive(Debug)]
pub struct Registry {
    defs: Vec<NodeDef>,
    by_kind: HashMap<String, usize>,
}

impl Registry {
    fn new(defs: Vec<NodeDef>) -> Self {
        let mut by_kind = HashMap::new();
        for (i, def) in defs.iter().enumerate() {
            assert!(
                by_kind.insert(def.kind.to_string(), i).is_none(),
                "Duplicate node type: {}",
                def.kind
            );
        }
        Self { defs, by_kind }
    }

    pub fn get(&self, kind: &str) -> Option<&NodeDef> {
        self.by_kind.get(kind).map(|i| &self.defs[*i])
    }

    pub fn all(&self) -> &[NodeDef] {
        &self.defs
    }

    pub fn visible(&self) -> impl Iterator<Item = &NodeDef> {
        self.defs.iter().filter(|d| !d.hidden)
    }
}

static REGISTRY: LazyLock<Registry> = LazyLock::new(|| Registry::new(nodes::all()));

pub fn registry() -> &'static Registry {
    &REGISTRY
}

pub fn node_def(kind: &str) -> Option<&'static NodeDef> {
    REGISTRY.get(kind)
}

pub fn is_output_type(kind: &str) -> bool {
    node_def(kind).is_some_and(|d| d.category == Category::Output)
}

pub fn default_params(kind: &str) -> Values {
    node_def(kind).map(NodeDef::defaults).unwrap_or_default()
}

pub fn custom_label(node: &WorkflowNode) -> Option<&str> {
    node.label
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
}

pub fn node_name(node: &WorkflowNode) -> String {
    match custom_label(node) {
        Some(label) => label.to_owned(),
        None => node_def(&node.kind)
            .map(|d| d.title.clone())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| node.kind.clone()),
    }
}
