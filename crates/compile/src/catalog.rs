use indexmap::IndexMap;
use opsis_types::actions::Viewport;
use opsis_types::fonts::FontFamilyDef;
use opsis_types::looks::NodeLook;
use opsis_types::mix::{MixerBook, mixer_book};
use opsis_types::settings::{SettingsBook, settings_book};
use opsis_types::terminal::TerminalTheme;
use opsis_types::timeline::Ease;
use opsis_types::workflow::{PortType, Theme, Workflow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::envelope::{EnvelopeTrack, envelope_tracks};
use crate::fonts::bundled_fonts;
use crate::node::{Category, NodeDef, Phase, Platform, PortSpec, Ports};
use crate::nodes::media::{loudness_presets, quality_presets};
use crate::nodes::output::{CANVAS_SHORT_SIDE, Variant, variant_canvas};
use crate::nodes::transitions::{transition_kinds, transition_title};
use crate::params::Values;
use crate::ports::can_connect;
use crate::registry::{node_def, registry};
use crate::size::estimate_card_height;
use crate::sounds::{builtin_songs, builtin_sounds};
use crate::timeline::camera_path::{CAMERA_DIALS, CameraDial};
use crate::vars::{PAUSE_MARK, VAR_PATTERN};

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, specta::Type,
)]
#[schemars(rename = "Port")]
pub struct CatalogPort {
    #[serde(rename = "type")]
    pub kind: PortType,
    pub label: String,
    pub optional: bool,
    #[schemars(description = "Takes any number of connections")]
    pub many: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema, specta::Type)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<CatalogFamily>,
    pub outputs: IndexMap<String, CatalogPort>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema, specta::Type)]
#[serde(rename_all = "camelCase")]
#[schemars(
    description = "Inputs that grow: <prefix>1, <prefix>2, … with one empty input after the last connected one"
)]
pub struct CatalogFamily {
    pub prefix: String,
    #[schemars(description = "What the numbered inputs are called together")]
    pub label: String,
    #[serde(rename = "type")]
    pub kind: PortType,
    #[schemars(description = "How many of the first inputs must be connected")]
    pub required: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema, specta::Type)]
pub struct CatalogEntry {
    #[serde(flatten)]
    pub node: CatalogNode,
    #[schemars(description = "JSON Schema of the node's settings")]
    #[specta(type = opsis_types::JsonObject)]
    pub params: Value,
}

const fn catalog_port(label: String, p: &PortSpec) -> CatalogPort {
    CatalogPort {
        kind: p.kind,
        label,
        optional: p.optional,
        many: p.many,
    }
}

fn ports(list: &Ports) -> IndexMap<String, CatalogPort> {
    list.iter()
        .map(|(id, p)| ((*id).to_owned(), catalog_port(p.label(), p)))
        .collect()
}

fn input_ports<'a>(
    def: &NodeDef,
    connected: impl IntoIterator<Item = &'a str>,
) -> IndexMap<String, CatalogPort> {
    def.input_ports(connected)
        .into_iter()
        .map(|(id, label, p)| (id, catalog_port(label, &p)))
        .collect()
}

fn catalog_family(def: &NodeDef) -> Option<CatalogFamily> {
    def.family.as_ref().map(|f| CatalogFamily {
        prefix: f.prefix.to_owned(),
        label: f.name.to_owned(),
        kind: f.kind(),
        required: f.required,
    })
}

pub fn node_inputs(wf: &Workflow) -> IndexMap<String, IndexMap<String, CatalogPort>> {
    wf.nodes
        .iter()
        .filter_map(|node| {
            let def = node_def(&node.kind).filter(|d| d.family.is_some())?;
            let handles = wf
                .edges
                .iter()
                .filter(|e| e.target == node.id)
                .map(|e| e.target_handle.as_str());
            Some((node.id.clone(), input_ports(def, handles)))
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
        inputs: input_ports(def, std::iter::empty()),
        family: catalog_family(def),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<CatalogFamily>,
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
    pub uses_project_size: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub look: Option<NodeLook>,
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
        inputs: input_ports(def, std::iter::empty()),
        family: catalog_family(def),
        outputs: ports(&def.outputs),
        params: def.params.json().clone(),
        defaults: def.defaults(),
        host_defaults: def.host_defaults.map_or_default(|f| f(platform)),
        is_virtual: def.is_virtual,
        action: def.action,
        executes: def.executes,
        hidden: def.hidden,
        uses_project_size: def.uses_project_size,
        element: def.element.clone(),
        look: def.look.clone(),
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
    pub theme_backgrounds: IndexMap<Theme, String>,
    pub shapes: Vec<ShapeChoice>,
    pub quality_presets: Vec<(String, f64)>,
    pub loudness_presets: Vec<(String, f64)>,
    pub timeline: TimelineBook,
    pub sounds: Vec<SoundChoice>,
    pub music: Vec<SoundChoice>,
    pub terminal_themes: Vec<TerminalTheme>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SoundChoice {
    pub reference: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TimelineBook {
    pub eases: Vec<(Ease, String)>,
    pub envelopes: Vec<EnvelopeTrack>,
    pub camera_dials: Vec<CameraDial>,
}

fn timeline_book() -> TimelineBook {
    TimelineBook {
        eases: Ease::ALL
            .into_iter()
            .map(|e| (e, e.label().to_owned()))
            .collect(),
        envelopes: envelope_tracks(),
        camera_dials: CAMERA_DIALS.to_vec(),
    }
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct ShapeChoice {
    pub shape: Variant,
    pub canvas: Viewport,
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
        theme_backgrounds: Theme::ALL
            .into_iter()
            .map(|theme| (theme, theme.background().to_owned()))
            .collect(),
        shapes: Variant::ALL
            .into_iter()
            .map(|shape| {
                let size = variant_canvas(shape, CANVAS_SHORT_SIDE);
                ShapeChoice {
                    shape,
                    canvas: Viewport {
                        width: size.width,
                        height: size.height,
                    },
                }
            })
            .collect(),
        quality_presets: quality_presets(),
        loudness_presets: loudness_presets(),
        timeline: timeline_book(),
        sounds: builtin_sounds()
            .iter()
            .map(|s| SoundChoice {
                reference: s.reference(),
                label: s.label.clone(),
                hint: None,
                duration_ms: s.duration_ms,
            })
            .collect(),
        terminal_themes: crate::terminal::theme_names()
            .iter()
            .filter_map(|name| crate::terminal::theme_named(name))
            .collect(),
        music: builtin_songs()
            .iter()
            .map(|s| SoundChoice {
                reference: s.reference(),
                label: s.label.clone(),
                hint: Some(s.mood.label().to_owned()),
                duration_ms: s.duration_ms,
            })
            .collect(),
    }
}
