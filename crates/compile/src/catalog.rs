use indexmap::IndexMap;
use opsis_types::fonts::FontFamilyDef;
use opsis_types::mix::{MixerBook, mixer_book};
use opsis_types::settings::{SettingsBook, settings_book};
use opsis_types::workflow::PortType;
use serde::Serialize;
use serde_json::Value;

use crate::fonts::bundled_fonts;
use crate::node::{Category, NodeDef, Phase, Platform, Ports};
use crate::nodes::transitions::{transition_kinds, transition_title};
use crate::params::Values;
use crate::ports::can_connect;
use crate::registry::{node_def, registry};
use crate::size::estimate_card_height;
use crate::vars::{PAUSE_MARK, VAR_PATTERN};

#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema, specta::Type)]
#[schemars(rename = "Port")]
pub struct CatalogPort {
    #[serde(rename = "type")]
    pub kind: PortType,
    pub label: String,
    pub optional: bool,
    #[schemars(description = "Takes any number of connections")]
    pub many: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema, specta::Type)]
#[schemars(rename = "NodeType")]
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

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EditorNode {
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub category: Category,
    pub category_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub phase: Phase,
    pub inputs: IndexMap<String, CatalogPort>,
    pub outputs: IndexMap<String, CatalogPort>,
    #[specta(type = opsis_types::JsonObject)]
    pub params: Value,
    #[specta(type = opsis_types::JsonObject)]
    pub defaults: Values,
    #[specta(type = opsis_types::JsonObject)]
    pub host_defaults: Values,
    #[serde(rename = "virtual")]
    pub is_virtual: bool,
    pub action: bool,
    pub executes: bool,
    pub hidden: bool,
    pub secret_params: Vec<String>,
    pub uses_project_size: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    pub card_height: f64,
    pub has_card: bool,
    pub compiles: bool,
}

fn editor_node(def: &NodeDef, platform: Platform) -> EditorNode {
    EditorNode {
        kind: def.kind.to_string(),
        title: def.title.clone(),
        category: def.category,
        category_label: def.category.label().to_owned(),
        description: def.description.clone(),
        phase: def.phase,
        inputs: ports(&def.inputs),
        outputs: ports(&def.outputs),
        params: def.params.json().clone(),
        defaults: def.defaults(),
        host_defaults: def.host_defaults.map(|f| f(platform)).unwrap_or_default(),
        is_virtual: def.is_virtual,
        action: def.action,
        executes: def.executes,
        hidden: def.hidden,
        secret_params: def.secret_params.iter().map(|p| (*p).to_owned()).collect(),
        uses_project_size: def.uses_project_size,
        element: def.element.clone(),
        card_height: estimate_card_height(Some(def)),
        has_card: def.has_card(),
        compiles: def.has_compile(),
    }
}

pub fn editor_catalog(platform: Platform) -> Vec<EditorNode> {
    registry()
        .all()
        .iter()
        .map(|def| editor_node(def, platform))
        .collect()
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct PortLink {
    pub from: PortType,
    pub to: PortType,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TextRules {
    pub variable_pattern: String,
    pub pause_mark: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EditorCatalog {
    pub nodes: Vec<EditorNode>,
    pub transitions: IndexMap<String, String>,
    pub connects: Vec<PortLink>,
    pub text: TextRules,
    pub mixer: MixerBook,
    pub bundled_fonts: Vec<FontFamilyDef>,
    pub settings: SettingsBook,
}

pub fn editor_bundle(platform: Platform) -> EditorCatalog {
    let transitions = transition_kinds()
        .into_iter()
        .map(|kind| (kind.clone(), transition_title(&kind)))
        .collect();
    let connects = PortType::ALL
        .iter()
        .flat_map(|from| {
            PortType::ALL
                .iter()
                .filter(|to| can_connect(*from, **to))
                .map(|to| PortLink {
                    from: *from,
                    to: *to,
                })
        })
        .collect();
    EditorCatalog {
        nodes: editor_catalog(platform),
        transitions,
        connects,
        text: TextRules {
            variable_pattern: VAR_PATTERN.to_owned(),
            pause_mark: PAUSE_MARK.to_string(),
        },
        mixer: mixer_book(),
        bundled_fonts: bundled_fonts().to_vec(),
        settings: settings_book(),
    }
}
