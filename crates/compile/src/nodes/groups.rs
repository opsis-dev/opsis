use std::sync::LazyLock;

use opsis_types::cards::{CardRow, Icon};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::values::CompileValue;
use opsis_types::workflow::{PortType, Workflow};
use serde_json::{Map, Value, json};

use crate::elements::introspect::shown;
use crate::elements::{ElementKind, ElementMeta, element_meta};
use crate::node::{CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Phase, outputs, port, row};
use crate::nodes::device_frame::drawn_elements;
use crate::nodes::elements::{
    CLIP_DURATION, ENVELOPE, EnvelopeField, clip_duration_field, compile_element, element_card,
    properties_of,
};
use crate::nodes::media::CLIP_ELEMENTS;
use crate::params::{ParamSchema, Values, schema_of};
use crate::registry::node_def;

pub const KIND: &str = "kind";
pub const KIND_OBJECT: &str = "x-kind";
const ZERO_LABEL: &str = "x-zero-label";
const USUAL_LENGTH: &str = "Its usual length";

#[derive(Debug, Clone)]
pub struct GroupKind {
    pub id: &'static str,
    pub label: String,
    pub element: &'static str,
    pub fixed: Values,
    pub hidden: Vec<&'static str>,
}

impl GroupKind {
    pub fn new(id: &'static str, label: impl Into<String>, element: &'static str) -> Self {
        Self {
            id,
            label: label.into(),
            element,
            fixed: Values::new(),
            hidden: Vec::new(),
        }
    }

    #[must_use]
    pub fn fixed(mut self, prop: &str, value: Value) -> Self {
        self.fixed.insert(prop.to_owned(), value);
        self
    }

    #[must_use]
    pub fn hide(mut self, props: &[&'static str]) -> Self {
        self.hidden.extend_from_slice(props);
        self
    }

    pub fn meta(&self) -> &'static ElementMeta {
        element_meta(self.element).unwrap_or_else(|| panic!("no element {}", self.element))
    }
}

#[derive(Debug)]
pub struct KindGroup {
    pub node_type: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub category: Category,
    pub kind_label: &'static str,
    pub kinds: Vec<GroupKind>,
    pub shared: Vec<&'static str>,
    pub look: NodeLook,
}

impl KindGroup {
    pub fn picked(&self, values: &Values) -> &GroupKind {
        let id = values.get(KIND).and_then(Value::as_str);
        self.kinds
            .iter()
            .find(|k| Some(k.id) == id)
            .unwrap_or(&self.kinds[0])
    }

    pub fn values_for(&self, kind: &GroupKind, values: &Values) -> Values {
        kind_values(kind, values, &self.shared)
    }
}

pub fn kind_values(kind: &GroupKind, values: &Values, shared: &[&str]) -> Values {
    let mut merged = kind.meta().default_props.clone();
    merged.extend(kind.fixed.clone());
    for key in shared {
        if let Some(value) = values.get(*key).filter(|v| fits(kind, key, v)) {
            merged.insert((*key).to_owned(), value.clone());
        }
    }
    if let Some(Value::Object(own)) = values.get(kind.id) {
        merged.extend(
            own.iter()
                .filter(|(key, _)| !kind.fixed.contains_key(*key))
                .map(|(k, v)| (k.clone(), v.clone())),
        );
    }
    for key in [CLIP_DURATION, ENVELOPE] {
        if let Some(value) = values.get(key) {
            merged.insert(key.to_owned(), value.clone());
        }
    }
    merged
}

fn fits(kind: &GroupKind, key: &str, value: &Value) -> bool {
    let Some(field) = kind.meta().schema.field(key) else {
        return false;
    };
    field
        .get("enum")
        .and_then(Value::as_array)
        .is_none_or(|allowed| allowed.contains(value))
}

fn without_fixed(properties: Map<String, Value>, fixed: &Values) -> Map<String, Value> {
    properties
        .into_iter()
        .filter(|(key, _)| !fixed.contains_key(key))
        .filter_map(|(key, mut field)| {
            let Some(rules) = field.get("x-show-if").and_then(Value::as_object).cloned() else {
                return Some((key, field));
            };
            let mut kept = Map::new();
            for (on, allowed) in rules {
                if fixed.contains_key(&on) {
                    let rule = json!({ "x-show-if": { on: allowed } });
                    if !shown(&rule, fixed) {
                        return None;
                    }
                } else {
                    kept.insert(on, allowed);
                }
            }
            if let Some(object) = field.as_object_mut() {
                if kept.is_empty() {
                    object.remove("x-show-if");
                } else {
                    object.insert("x-show-if".to_owned(), Value::Object(kept));
                }
            }
            Some((key, field))
        })
        .collect()
}

pub fn kind_object(kind: &GroupKind, shared: &[&str]) -> Value {
    kind_object_shown(kind, shared, &json!({ KIND: [kind.id] }))
}

pub fn kind_object_shown(kind: &GroupKind, shared: &[&str], show_if: &Value) -> Value {
    let mut properties = properties_of(kind.meta().schema.json());
    properties
        .retain(|key, _| !shared.contains(&key.as_str()) && !kind.hidden.contains(&key.as_str()));
    json!({
        "type": "object",
        "properties": without_fixed(properties, &kind.fixed),
        KIND_OBJECT: true,
        "x-show-if": show_if,
    })
}

pub fn kind_choice(label: &str, kinds: &[GroupKind]) -> Value {
    let ids: Vec<&str> = kinds.iter().map(|k| k.id).collect();
    let labels: Map<String, Value> = kinds
        .iter()
        .map(|k| (k.id.to_owned(), Value::from(k.label.as_str())))
        .collect();
    json!({
        "description": label,
        "type": "string",
        "enum": ids,
        "default": ids.first(),
        "x-labels": labels,
    })
}

pub fn kind_properties(label: &str, kinds: &[GroupKind], shared: &[&str]) -> Map<String, Value> {
    let mut properties = Map::new();
    properties.insert(KIND.to_owned(), kind_choice(label, kinds));
    for kind in kinds {
        properties.insert(kind.id.to_owned(), kind_object(kind, shared));
    }
    for key in shared {
        let having: Vec<&str> = kinds
            .iter()
            .filter(|k| k.meta().schema.field(key).is_some())
            .map(|k| k.id)
            .collect();
        let Some(mut field) = kinds
            .iter()
            .find_map(|k| k.meta().schema.field(key).cloned())
        else {
            continue;
        };
        if having.len() < kinds.len()
            && let Some(object) = field.as_object_mut()
        {
            let mut rules = object
                .get("x-show-if")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            rules.insert(KIND.to_owned(), json!(having));
            object.insert("x-show-if".to_owned(), Value::Object(rules));
        }
        properties.insert((*key).to_owned(), field);
    }
    properties
}

fn duration_field(group: &KindGroup) -> Value {
    let mut field = clip_duration_field(group.kinds[0].meta());
    let labels: Vec<Option<Value>> = group
        .kinds
        .iter()
        .map(|k| clip_duration_field(k.meta()).get(ZERO_LABEL).cloned())
        .collect();
    let label = match labels.first() {
        Some(Some(first)) if labels.iter().all(|l| l.as_ref() == Some(first)) => first.clone(),
        _ => Value::from(USUAL_LENGTH),
    };
    field["minimum"] = Value::from(0);
    field["default"] = Value::from(0);
    field[ZERO_LABEL] = label;
    field
}

fn group_schema(group: &KindGroup) -> ParamSchema {
    let mut properties = kind_properties(group.kind_label, &group.kinds, &group.shared);
    properties.insert(CLIP_DURATION.to_owned(), duration_field(group));
    properties.extend(properties_of(&schema_of::<EnvelopeField>()));
    ParamSchema::new(json!({ "type": "object", "properties": properties }))
}

fn kind_row(kind: &GroupKind) -> CardRow {
    row(Icon::Shapes, kind.label.as_str())
}

pub fn group_node(group: &'static KindGroup) -> NodeDef {
    let metas: Vec<&ElementMeta> = group.kinds.iter().map(GroupKind::meta).collect();
    let wraps = metas.iter().any(|m| m.kind == ElementKind::Wrapper);
    let clip_optional = metas
        .iter()
        .any(|m| m.kind != ElementKind::Wrapper || m.optional_clip);
    let inputs = if wraps {
        let clip = port(PortType::Clip, "Clip");
        vec![(
            CLIP_PORT,
            if clip_optional { clip.optional() } else { clip },
        )]
    } else {
        Vec::new()
    };
    NodeDef::new(
        group.node_type,
        group.title,
        group.category,
        Phase::Compile,
        group_schema(group),
    )
    .describe(group.description)
    .look(group.look.clone())
    .inputs(inputs)
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(move |ctx: &CardCtx<'_, Values>| {
        let kind = group.picked(ctx.values);
        let values = group.values_for(kind, ctx.values);
        std::iter::once(kind_row(kind))
            .chain(element_card(
                kind.meta(),
                &values,
                ctx.fps(),
                ctx.connected(CLIP_PORT),
            ))
            .map(Some)
            .collect()
    })
    .compiles(move |ctx: &Ctx<'_, Values>| {
        let kind = group.picked(ctx.values);
        let values = group.values_for(kind, ctx.values);
        let input = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip);
        outputs([(
            CLIP_PORT,
            CompileValue::Clip(compile_element(
                kind.meta(),
                ctx.node_id,
                &values,
                input,
                ctx.settings,
            )),
        )])
    })
}

fn other_kinds(params: &ParamSchema, values: &Values) -> Vec<String> {
    let current = params.with_defaults(values);
    params
        .json()
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(_, field)| field.get(KIND_OBJECT) == Some(&Value::Bool(true)))
        .filter(|(_, field)| !shown(field, &current))
        .map(|(key, _)| key.clone())
        .collect()
}

pub fn saved_workflow(workflow: &Workflow) -> Workflow {
    let mut saved = workflow.clone();
    for node in &mut saved.nodes {
        let Some(def) = node_def(&node.kind) else {
            continue;
        };
        for key in other_kinds(&def.params, &node.params) {
            node.params.remove(&key);
        }
    }
    saved
}

pub fn grouped_elements() -> &'static [&'static str] {
    &GROUPED
}

pub fn groups() -> &'static [KindGroup] {
    &GROUPS
}

fn kind(id: &'static str, label: &'static str, element: &'static str) -> GroupKind {
    GroupKind::new(id, label, element)
}

fn animated_text() -> KindGroup {
    KindGroup {
        node_type: "fx.animated-text",
        title: "Animated text",
        description: "A line of text that comes in with the animation you pick.",
        category: Category::TextAndOverlays,
        kind_label: "Animation",
        kinds: vec![
            kind("soft-blur", "Soft blur in", "soft-blur-in"),
            kind("typewriter", "Typewriter", "typewriter"),
            kind("fade-up", "Staggered fade up", "staggered-fade-up"),
            kind("word-push", "Word push", "word-push"),
            kind("mask-reveal", "Mask reveal up", "mask-reveal-up"),
            kind("tracking", "Tracking in", "tracking-in"),
            kind("glitch", "Colour glitch", "rgb-glitch-text"),
            kind("decode", "Decode", "matrix-decode"),
            kind("line-by-line", "Line by line slide", "line-by-line-slide"),
            kind("headline", "Animated headline", "kinetic-headline"),
            kind("handwriting", "Handwriting", "handwriting-text"),
            kind("neon", "Neon sign", "neon-flicker-text"),
            kind("scale-cut", "Gradient scale cut", "gradient-scale-cut-text"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:type").moves(NodeMotion::BlurSwap),
    }
}

fn arrow() -> KindGroup {
    KindGroup {
        node_type: "fx.arrow",
        title: "Arrow",
        description: "An arrow that draws itself to point at something, clean with a label or hand-drawn with a brush.",
        category: Category::TextAndOverlays,
        kind_label: "Look",
        kinds: vec![
            kind("clean", "Clean", "callout"),
            kind("brush", "Brush", "ink-arrow"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("tabler:arrow-curve-right").moves(NodeMotion::Draw),
    }
}

fn background() -> KindGroup {
    KindGroup {
        node_type: "fx.background",
        title: "Background",
        description: "A background to put behind other layers: a colour, a gradient, a texture or a moving pattern.",
        category: Category::Backgrounds,
        kind_label: "Kind",
        kinds: vec![
            kind("solid", "Solid colour", "solid"),
            kind("gradient", "Gradient", "gradient"),
            kind("paper", "Paper", "paper"),
            kind("studio", "Studio backdrop", "backdrop"),
            kind("noise", "Noise", "noise"),
            kind("starburst", "Starburst", "starburst"),
            kind("aurora", "Aurora", "aurora-bg"),
            kind("particles", "Particle field", "particle-field"),
            kind("shapes", "Floating shapes", "shapes-3d"),
            kind("mesh", "Mesh gradient", "shader-mesh-gradient"),
            kind("grain", "Grain gradient", "shader-grain-gradient"),
            kind("warp", "Warp", "shader-warp"),
            kind("swirl", "Swirl", "shader-swirl"),
            kind("rays", "Light rays", "shader-god-rays"),
            kind("clouds", "Clouds", "shader-perlin-noise"),
            kind("dither", "Dither", "shader-dithering"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("tabler:background"),
    }
}

fn logo_set() -> KindGroup {
    KindGroup {
        node_type: "fx.logo-set",
        title: "Logo set",
        description: "A set of logos shown together: a grid that lights up, a field drifting past a headline, or a stack of chips.",
        category: Category::Elements,
        kind_label: "Layout",
        kinds: vec![
            kind("grid", "Grid", "logo-wall"),
            kind("drift", "Drift", "logo-drift"),
            kind("stack", "Stack", "logo-enter"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:layout-grid"),
    }
}

fn logo_reveal() -> KindGroup {
    KindGroup {
        node_type: "fx.logo-reveal",
        title: "Logo reveal",
        description: "Brings in your logo and name: drawn as an outline, assembled from orbiting cards, or wiped in behind a spinning mark.",
        category: Category::Elements,
        kind_label: "Style",
        kinds: vec![
            kind("outline", "Outline", "logo-reveal"),
            kind("orbit", "Orbit", "logo-assemble"),
            kind("wipe", "Wipe", "logo-wipe"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:pen-tool").moves(NodeMotion::Draw),
    }
}

fn code() -> KindGroup {
    KindGroup {
        node_type: "fx.code-reveal",
        title: "Code reveal",
        description: "A code editor that writes out a file, then highlights the lines that matter.",
        category: Category::Elements,
        kind_label: "Look",
        kinds: vec![
            kind("plain", "Plain", "code-reveal"),
            kind("glass", "Glass", "glass-code-block"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:code"),
    }
}

fn code_diff() -> KindGroup {
    KindGroup {
        node_type: "fx.code-diff-wipe",
        title: "Code diff",
        description: "A code editor that changes one version of a file into another.",
        category: Category::Elements,
        kind_label: "Style",
        kinds: vec![
            kind("line-wipe", "Line wipe", "code-diff-wipe"),
            kind("token-morph", "Token morph", "code-morph"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:file-diff"),
    }
}

fn chart() -> KindGroup {
    KindGroup {
        node_type: "fx.chart",
        title: "Chart",
        description: "A chart that animates in from your numbers.",
        category: Category::Elements,
        kind_label: "Type",
        kinds: vec![
            kind("bar", "Bar chart", "animated-bar-chart"),
            kind("line", "Line chart", "animated-line-chart"),
            kind("funnel", "Funnel", "funnel-chart"),
            kind("donut", "Donut", "donut-chart"),
            kind("before-after", "Before and after bars", "comparison-bars"),
            kind("race", "Bar chart race", "bar-chart-race"),
            kind("gauge", "Gauge", "gauge-dial"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:chart-column").moves(NodeMotion::Grow),
    }
}

fn metric_cards() -> KindGroup {
    KindGroup {
        node_type: "fx.metric-ticker",
        title: "Metric cards",
        description: "Key numbers that count up: several on cards with their trends, or one in a filling ring.",
        category: Category::Elements,
        kind_label: "Style",
        kinds: vec![
            kind("cards", "Cards", "metric-ticker"),
            kind("ring", "Ring", "stat-card"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:trending-up"),
    }
}

fn rolling_number() -> KindGroup {
    KindGroup {
        node_type: "fx.rolling-number",
        title: "Rolling number",
        description: "A value that rolls to its target: digits like an odometer, or a reel that rolls any characters.",
        category: Category::Elements,
        kind_label: "Style",
        kinds: vec![
            kind("odometer", "Odometer", "rolling-number"),
            kind("reel", "Reel", "slot-machine-roll"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:hash").moves(NodeMotion::Roll),
    }
}

fn film_look() -> KindGroup {
    KindGroup {
        node_type: "fx.film-look",
        title: "Film look",
        description: "Shows a clip as if seen through something: an old screen, a tape, a printed page or a camera lens.",
        category: Category::Effects,
        kind_label: "Look",
        kinds: vec![
            kind("crt", "Old screen", "crt-screen"),
            kind("tape", "Tape", "vhs-filter"),
            kind("print", "Halftone print", "halftone-print"),
            kind("lens", "Camera lens", "camera-lens"),
        ],
        shared: Vec::new(),
        look: NodeLook::of("lucide:tv-minimal"),
    }
}

fn object_3d() -> KindGroup {
    KindGroup {
        node_type: "fx.object-3d",
        title: "3D object",
        description: "A logo, an image card, text or a model in 3D, turning under studio lights.",
        category: Category::Elements,
        kind_label: "What",
        kinds: vec![
            kind("logo", "Logo", "logo-3d"),
            kind("card", "Image card", "card-3d"),
            kind("text", "Text", "text-3d"),
            kind("model", "Model", "model-3d"),
        ],
        shared: vec![
            "camera",
            "speed",
            "zoom",
            "growIn",
            "endFacing",
            "motionAmount",
            "reverse",
            "lens",
        ],
        look: NodeLook::of("lucide:cuboid").moves(NodeMotion::Turn),
    }
}

static GROUPS: LazyLock<Vec<KindGroup>> = LazyLock::new(|| {
    vec![
        animated_text(),
        arrow(),
        background(),
        logo_set(),
        logo_reveal(),
        code(),
        code_diff(),
        chart(),
        metric_cards(),
        rolling_number(),
        film_look(),
        object_3d(),
    ]
});

static GROUPED: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    GROUPS
        .iter()
        .flat_map(|g| g.kinds.iter().map(|k| k.element))
        .chain(drawn_elements())
        .chain(CLIP_ELEMENTS)
        .collect()
});
