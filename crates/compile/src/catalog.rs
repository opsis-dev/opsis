use indexmap::IndexMap;
use opsis_types::workflow::PortType;
use serde::Serialize;
use serde_json::Value;

use crate::node::{NodeDef, Ports};
use crate::registry::{node_def, registry};

#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema, specta::Type)]
pub struct CatalogPort {
    #[serde(rename = "type")]
    pub kind: PortType,
    pub label: String,
    pub optional: bool,
    #[schemars(description = "Takes any number of connections")]
    pub many: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema, specta::Type)]
pub struct CatalogNode {
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub category: String,
    pub description: String,
    #[schemars(description = "Does work in a run (records, exports)")]
    pub runs: bool,
    pub inputs: IndexMap<String, CatalogPort>,
    pub outputs: IndexMap<String, CatalogPort>,
}

#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema, specta::Type)]
pub struct CatalogEntry {
    #[serde(flatten)]
    pub node: CatalogNode,
    #[schemars(description = "JSON Schema of the node's settings")]
    #[specta(type = opsis_types::JsonObject)]
    pub params: Value,
}

fn ports(list: &Ports) -> IndexMap<String, CatalogPort> {
    list.iter()
        .map(|(id, p)| {
            (
                (*id).to_owned(),
                CatalogPort {
                    kind: p.kind,
                    label: p.label(),
                    optional: p.optional,
                    many: p.many,
                },
            )
        })
        .collect()
}

fn catalog_node(def: &NodeDef) -> CatalogNode {
    CatalogNode {
        kind: def.kind.to_string(),
        title: def.title.clone(),
        category: def.category.label().to_owned(),
        description: def.description.clone().unwrap_or_default(),
        runs: def.executes || def.is_virtual,
        inputs: ports(&def.inputs),
        outputs: ports(&def.outputs),
    }
}

pub fn node_catalog() -> Vec<CatalogNode> {
    registry().visible().map(catalog_node).collect()
}

pub fn catalog_entry(kind: &str) -> Option<CatalogEntry> {
    let def = node_def(kind)?;
    Some(CatalogEntry {
        node: catalog_node(def),
        params: def.params.json().clone(),
    })
}
