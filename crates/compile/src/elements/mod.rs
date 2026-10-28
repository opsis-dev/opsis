pub mod choices;
pub mod core;
pub mod default_props;
pub mod entry;
pub mod introspect;
pub mod length_cases;
pub mod lengths;
pub mod prepared;
pub mod remocn;
pub mod remotion_ui;
pub mod snapcn;
pub mod theme;

use std::collections::HashMap;
use std::sync::LazyLock;

use indexmap::IndexMap;

use opsis_types::looks::NodeLook;
use opsis_types::timeline::Design;
use serde::{Deserialize, Serialize};

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
    pub kind_props: IndexMap<String, String>,
    pub default_duration_in_frames: u32,
    pub design: Option<Design>,
    pub optional_clip: bool,
    pub thumbnail: Option<String>,
    pub look: Option<NodeLook>,
}

struct Elements {
    list: Vec<ElementMeta>,
    by_key: HashMap<String, usize>,
}

static ELEMENTS: LazyLock<Elements> = LazyLock::new(|| {
    let list: Vec<ElementMeta> = [
        core::core_elements(),
        remocn::elements(),
        snapcn::elements(),
        remotion_ui::elements(),
    ]
    .concat();
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
