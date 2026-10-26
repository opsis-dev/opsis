pub mod core;
pub mod introspect;
pub mod length_cases;
pub mod lengths;
pub mod prepared;

use std::collections::HashMap;
use std::sync::LazyLock;

use opsis_types::timeline::Design;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::params::{ParamSchema, Values};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementCategory {
    Core,
    Text,
    Overlay,
    Transition,
    Background,
    Effect,
    Ui,
    Data,
    Scene,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementKind {
    Component,
    Wrapper,
    Presentation,
    Sequencer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementSource {
    pub registry: String,
    pub url: String,
    pub license: String,
}

#[derive(Debug, Clone)]
pub struct ElementMeta {
    pub key: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ElementCategory,
    pub kind: ElementKind,
    pub source: ElementSource,
    pub schema: ParamSchema,
    pub default_props: Values,
    pub fixed_props: Values,
    pub default_duration_in_frames: u32,
    pub design: Option<Design>,
    pub optional_clip: bool,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ElementEntry {
    key: String,
    title: String,
    #[serde(default)]
    description: Option<String>,
    category: ElementCategory,
    kind: ElementKind,
    source: ElementSource,
    schema: Value,
    default_props: Values,
    fixed_props: Values,
    default_duration_in_frames: u32,
    #[serde(default)]
    design: Option<Design>,
    #[serde(default)]
    optional_clip: bool,
    #[serde(default)]
    thumbnail: Option<String>,
}

impl From<ElementEntry> for ElementMeta {
    fn from(e: ElementEntry) -> Self {
        Self {
            key: e.key,
            title: e.title,
            description: e.description,
            category: e.category,
            kind: e.kind,
            source: e.source,
            schema: ParamSchema::new(e.schema),
            default_props: e.default_props,
            fixed_props: e.fixed_props,
            default_duration_in_frames: e.default_duration_in_frames,
            design: e.design,
            optional_clip: e.optional_clip,
            thumbnail: e.thumbnail,
        }
    }
}

fn registry_elements() -> Vec<ElementMeta> {
    let entries: Vec<ElementEntry> = serde_json::from_str(include_str!("registry.generated.json"))
        .expect("registry element list parses");
    entries.into_iter().map(ElementMeta::from).collect()
}

struct Elements {
    list: Vec<ElementMeta>,
    by_key: HashMap<String, usize>,
}

static ELEMENTS: LazyLock<Elements> = LazyLock::new(|| {
    let list: Vec<ElementMeta> = core::core_elements()
        .into_iter()
        .chain(registry_elements())
        .collect();
    let by_key = list
        .iter()
        .enumerate()
        .map(|(i, e)| (e.key.clone(), i))
        .collect();
    Elements { list, by_key }
});

pub fn elements() -> &'static [ElementMeta] {
    &ELEMENTS.list
}

pub fn element_meta(key: &str) -> Option<&'static ElementMeta> {
    ELEMENTS.by_key.get(key).map(|i| &ELEMENTS.list[*i])
}

pub fn element_node_type(key: &str) -> String {
    format!("fx.{}", key.replacen('/', ".", 1))
}
