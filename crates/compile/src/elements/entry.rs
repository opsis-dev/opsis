use indexmap::IndexMap;
use opsis_types::looks::NodeLook;
use opsis_types::timeline::Design;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::elements::{ElementCategory, ElementKind, ElementMeta, ElementSource};
use crate::params::{ParamSchema, Values, to_values};

const OPSIS_URL: &str = "https://github.com/bismitpanda/opsis";

pub struct Entry {
    key: &'static str,
    title: &'static str,
    description: &'static str,
    category: ElementCategory,
    kind: ElementKind,
    frames: u32,
    look: Option<NodeLook>,
    source: Option<ElementSource>,
    design: Option<Design>,
    fixed_props: Values,
    kind_props: IndexMap<String, String>,
    optional_clip: bool,
}

impl Entry {
    #[must_use]
    pub fn look(mut self, look: NodeLook) -> Self {
        self.look = Some(look);
        self
    }

    #[must_use]
    pub fn source(mut self, registry: &str, url: &str, license: &str) -> Self {
        self.source = Some(ElementSource {
            registry: registry.to_owned(),
            url: url.to_owned(),
            license: license.to_owned(),
        });
        self
    }

    #[must_use]
    pub fn design(mut self, width: u32, height: u32) -> Self {
        self.design = Some(Design {
            width,
            height,
            exact: None,
        });
        self
    }

    #[must_use]
    pub fn exact_design(mut self, width: u32, height: u32) -> Self {
        self.design = Some(Design {
            width,
            height,
            exact: Some(true),
        });
        self
    }

    #[must_use]
    pub fn fixed(mut self, prop: &str, value: Value) -> Self {
        self.fixed_props.insert(prop.to_owned(), value);
        self
    }

    #[must_use]
    pub fn kind_prop(mut self, prop: &str, setting: &str) -> Self {
        self.kind_props.insert(prop.to_owned(), setting.to_owned());
        self
    }

    #[must_use]
    pub fn optional_clip(mut self) -> Self {
        self.optional_clip = true;
        self
    }
}

pub fn element(
    kind: ElementKind,
    key: &'static str,
    title: &'static str,
    description: &'static str,
    category: ElementCategory,
    frames: u32,
) -> Entry {
    Entry {
        key,
        title,
        description,
        category,
        kind,
        frames,
        look: None,
        source: None,
        design: None,
        fixed_props: Values::new(),
        kind_props: IndexMap::new(),
        optional_clip: false,
    }
}

pub fn component(
    key: &'static str,
    title: &'static str,
    description: &'static str,
    category: ElementCategory,
    frames: u32,
) -> Entry {
    element(
        ElementKind::Component,
        key,
        title,
        description,
        category,
        frames,
    )
}

pub fn wrapper(
    key: &'static str,
    title: &'static str,
    description: &'static str,
    category: ElementCategory,
    frames: u32,
) -> Entry {
    element(
        ElementKind::Wrapper,
        key,
        title,
        description,
        category,
        frames,
    )
}

pub fn presentation(key: &'static str, title: &'static str, frames: u32) -> Entry {
    element(
        ElementKind::Presentation,
        key,
        title,
        "",
        ElementCategory::Transition,
        frames,
    )
}

fn own_source(e: &Entry) -> ElementSource {
    if e.kind == ElementKind::Presentation {
        let slug = e.key.trim_start_matches("core/");
        ElementSource {
            registry: "@remotion/transitions".into(),
            url: format!("https://www.remotion.dev/docs/transitions/presentations/{slug}"),
            license: "Remotion".into(),
        }
    } else {
        ElementSource {
            registry: "Opsis".into(),
            url: OPSIS_URL.into(),
            license: "GPL-3.0-or-later".into(),
        }
    }
}

pub fn meta<T: JsonSchema + Default + Serialize>(e: Entry) -> ElementMeta {
    let source = e.source.clone().unwrap_or_else(|| own_source(&e));
    ElementMeta {
        key: e.key.to_owned(),
        title: e.title.to_owned(),
        description: (!e.description.is_empty()).then(|| e.description.to_owned()),
        category: e.category,
        kind: e.kind,
        source,
        schema: ParamSchema::of::<T>(),
        default_props: to_values(&T::default()),
        fixed_props: e.fixed_props,
        kind_props: e.kind_props,
        default_duration_in_frames: e.frames,
        design: e.design,
        optional_clip: e.optional_clip,
        thumbnail: None,
        look: e.look,
    }
}
