use std::borrow::Cow;
use std::sync::Arc;

use indexmap::IndexMap;
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::recording::RecordingProps;
use opsis_types::values::{
    ClipValue, CompileValue, EntriesValue, MediaKind, NodeResult, PortValues, StyleEntry,
};
use opsis_types::workflow::{PortType, Workflow, WorkflowSettings};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::params::{ParamSchema, Params, Values};
use crate::vars::Vars;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    specta::Type,
)]
pub enum Category {
    Browser,
    Capture,
    Camera,
    Cursor,
    #[serde(rename = "Text & overlays")]
    TextAndOverlays,
    Transitions,
    Backgrounds,
    Effects,
    Grade,
    Elements,
    Audio,
    Timeline,
    Media,
    Output,
    Utility,
}

impl Category {
    pub const ALL: [Category; 15] = [
        Self::Browser,
        Self::Capture,
        Self::Camera,
        Self::Cursor,
        Self::TextAndOverlays,
        Self::Transitions,
        Self::Backgrounds,
        Self::Effects,
        Self::Grade,
        Self::Elements,
        Self::Audio,
        Self::Timeline,
        Self::Media,
        Self::Output,
        Self::Utility,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Browser => "Browser",
            Self::Capture => "Record",
            Self::Camera => "Camera",
            Self::Cursor => "Cursor & clicks",
            Self::TextAndOverlays => "Text & overlays",
            Self::Transitions => "Transitions",
            Self::Backgrounds => "Backgrounds",
            Self::Effects => "Effects",
            Self::Grade => "Colour grade",
            Self::Elements => "Graphics",
            Self::Audio => "Sound",
            Self::Timeline => "Timeline",
            Self::Media => "Media",
            Self::Output => "Export",
            Self::Utility => "Organise",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Capture,
    Media,
    Compile,
    Render,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSpec {
    pub kind: PortType,
    pub label: Option<Cow<'static, str>>,
    pub optional: bool,
    pub many: bool,
}

impl PortSpec {
    pub fn label(&self) -> String {
        self.label
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| crate::text::humanize(self.kind.word()))
    }

    pub fn optional(self) -> Self {
        Self {
            optional: true,
            ..self
        }
    }

    pub fn many(self) -> Self {
        Self {
            optional: true,
            many: true,
            ..self
        }
    }
}

pub fn port(kind: PortType, label: impl Into<Cow<'static, str>>) -> PortSpec {
    PortSpec {
        kind,
        label: Some(label.into()),
        optional: false,
        many: false,
    }
}

pub fn opt(kind: PortType, label: impl Into<Cow<'static, str>>) -> PortSpec {
    port(kind, label).optional()
}

pub fn many(kind: PortType, label: impl Into<Cow<'static, str>>) -> PortSpec {
    port(kind, label).many()
}

pub type Ports = Vec<(&'static str, PortSpec)>;
pub type Inputs = PortValues;
pub type Outputs = PortValues;

pub const CLIP_PORT: &str = "clip";
pub const RECORDING_PORT: &str = "recording";
pub const STYLE_PORT: &str = "style";
pub const GRADE_PORT: &str = "grade";
pub const SESSION_PORT: &str = "session";

pub struct Ctx<'a, P> {
    pub node_id: &'a str,
    pub kind: &'a str,
    pub label: Option<&'a str>,
    pub params: P,
    pub values: &'a Values,
    pub inputs: &'a Inputs,
    pub settings: &'a WorkflowSettings,
    pub result: Option<&'a NodeResult>,
    pub vars: &'a Vars,
    pub workflow: &'a Workflow,
}

impl<P> Ctx<'_, P> {
    pub fn input(&self, port: &str) -> Option<&CompileValue> {
        self.inputs.get(port)
    }

    pub fn entry(&self) -> StyleEntry {
        StyleEntry {
            node_id: self.node_id.to_owned(),
            kind: self.kind.to_owned(),
            params: self.values.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputTake {
    pub duration_ms: f64,
    pub cursor_track: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputMedia {
    pub kind: MediaKind,
    pub has_audio: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct CardInputs {
    pub secrets: Option<Vec<String>>,
    pub connected: Vec<String>,
    pub clip_frames: Option<f64>,
    pub frames: IndexMap<String, f64>,
    pub items: IndexMap<String, f64>,
    pub takes: IndexMap<String, Option<InputTake>>,
    pub media: IndexMap<String, InputMedia>,
    pub captions_ms: IndexMap<String, f64>,
    pub steps: IndexMap<String, Vec<String>>,
    pub style: IndexMap<String, Vec<String>>,
    pub grade: IndexMap<String, Vec<StyleEntry>>,
    pub missing_files: Vec<String>,
}

impl CardInputs {
    pub fn is_connected(&self, port: &str) -> bool {
        self.connected.iter().any(|c| c == port)
    }
}

pub struct CardCtx<'a, P> {
    pub params: P,
    pub values: &'a Values,
    pub settings: &'a WorkflowSettings,
    pub result: Option<&'a NodeResult>,
    pub inputs: &'a CardInputs,
}

impl<P> CardCtx<'_, P> {
    pub fn fps(&self) -> f64 {
        self.settings.fps
    }

    pub fn connected(&self, port: &str) -> bool {
        self.inputs.is_connected(port)
    }
}

pub type Rows = Vec<Option<CardRow>>;
pub const MAX_CARD_ROWS: usize = 4;

type CompileFn = Arc<dyn Fn(&Ctx<'_, ()>) -> Result<Option<Outputs>, String> + Send + Sync>;
type CardFn = Arc<dyn Fn(&CardCtx<'_, ()>) -> Rows + Send + Sync>;
type MutatesFn = Arc<dyn Fn(&Values) -> Option<String> + Send + Sync>;
type HostDefaultsFn = fn(Platform) -> Values;
type CheckFn = Arc<dyn Fn(&Values) -> Result<(), String> + Send + Sync>;
type RestyleFn =
    Arc<dyn Fn(RecordingProps, &StyleEntry) -> Result<RecordingProps, String> + Send + Sync>;
type NeutralFn = Arc<dyn Fn(&Values) -> bool + Send + Sync>;
type RetimeFn = Arc<dyn Fn(ClipValue, &StyleEntry, f64) -> Result<ClipValue, String> + Send + Sync>;

#[derive(Debug, Clone, Copy)]
pub struct Retiming<'a> {
    pub node_id: &'a str,
    pub fps: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Mac,
    Windows,
}

#[derive(Clone)]
pub struct NodeDef {
    pub kind: Cow<'static, str>,
    pub title: String,
    pub category: Category,
    pub description: Option<String>,
    pub phase: Phase,
    pub inputs: Ports,
    pub outputs: Ports,
    pub params: ParamSchema,
    pub is_virtual: bool,
    pub action: bool,
    pub executes: bool,
    pub secret_params: &'static [&'static str],
    pub uses_project_size: bool,
    pub element: Option<String>,
    pub hidden: bool,
    pub card_height: Option<f64>,
    pub host_defaults: Option<HostDefaultsFn>,
    check: Option<CheckFn>,
    compile: Option<CompileFn>,
    card: Option<CardFn>,
    mutates: Option<MutatesFn>,
    restyle: Option<RestyleFn>,
    retime: Option<RetimeFn>,
    neutral: Option<NeutralFn>,
}

impl std::fmt::Debug for NodeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeDef")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

fn typed<P: DeserializeOwned>(values: &Values) -> Result<P, String> {
    crate::params::typed_values(values)
}

fn retyped<'a, P>(ctx: &Ctx<'a, ()>, params: P) -> Ctx<'a, P> {
    Ctx {
        node_id: ctx.node_id,
        kind: ctx.kind,
        label: ctx.label,
        params,
        values: ctx.values,
        inputs: ctx.inputs,
        settings: ctx.settings,
        result: ctx.result,
        vars: ctx.vars,
        workflow: ctx.workflow,
    }
}

impl NodeDef {
    pub fn new(
        kind: impl Into<Cow<'static, str>>,
        title: impl Into<String>,
        category: Category,
        phase: Phase,
        params: ParamSchema,
    ) -> Self {
        Self {
            kind: kind.into(),
            title: title.into(),
            category,
            description: None,
            phase,
            inputs: Vec::new(),
            outputs: Vec::new(),
            params,
            is_virtual: false,
            action: false,
            executes: false,
            secret_params: &[],
            uses_project_size: false,
            element: None,
            hidden: false,
            card_height: None,
            host_defaults: None,
            check: None,
            compile: None,
            card: None,
            mutates: None,
            restyle: None,
            retime: None,
            neutral: None,
        }
    }

    pub fn of<P: Params>(
        kind: impl Into<Cow<'static, str>>,
        title: impl Into<String>,
        category: Category,
        phase: Phase,
    ) -> Self {
        let mut def = Self::new(kind, title, category, phase, ParamSchema::of::<P>());
        def.check = Some(Arc::new(|values| typed::<P>(values)?.check()));
        def
    }

    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn inputs(mut self, ports: Ports) -> Self {
        self.inputs = ports;
        self
    }

    pub fn outputs(mut self, ports: Ports) -> Self {
        self.outputs = ports;
        self
    }

    pub fn virtual_node(mut self) -> Self {
        self.is_virtual = true;
        self
    }

    pub fn action(mut self) -> Self {
        self.action = true;
        self
    }

    pub fn executes(mut self) -> Self {
        self.executes = true;
        self
    }

    pub fn secrets(mut self, keys: &'static [&'static str]) -> Self {
        self.secret_params = keys;
        self
    }

    pub fn uses_project_size(mut self) -> Self {
        self.uses_project_size = true;
        self
    }

    pub fn element(mut self, key: impl Into<String>) -> Self {
        self.element = Some(key.into());
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn card_height(mut self, height: f64) -> Self {
        self.card_height = Some(height);
        self
    }

    pub fn host_defaults(mut self, f: HostDefaultsFn) -> Self {
        self.host_defaults = Some(f);
        self
    }

    pub fn compile<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&Ctx<'_, P>) -> Result<Option<Outputs>, String> + Send + Sync + 'static,
    {
        self.compile = Some(Arc::new(move |ctx| {
            f(&retyped(ctx, typed::<P>(ctx.values)?))
        }));
        self
    }

    pub fn compiles<P, F>(self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&Ctx<'_, P>) -> Outputs + Send + Sync + 'static,
    {
        self.compile(move |ctx: &Ctx<'_, P>| Ok(Some(f(ctx))))
    }

    pub fn card<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&CardCtx<'_, P>) -> Rows + Send + Sync + 'static,
    {
        self.card = Some(Arc::new(move |ctx| match typed::<P>(ctx.values) {
            Ok(params) => f(&CardCtx {
                params,
                values: ctx.values,
                settings: ctx.settings,
                result: ctx.result,
                inputs: ctx.inputs,
            }),
            Err(_) => Vec::new(),
        }));
        self
    }

    pub fn mutates<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&P) -> Option<String> + Send + Sync + 'static,
    {
        self.mutates = Some(Arc::new(move |values| {
            typed::<P>(values).ok().and_then(|p| f(&p))
        }));
        self
    }

    pub fn restyle<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(RecordingProps, &P, &str) -> RecordingProps + Send + Sync + 'static,
    {
        let schema = self.params.clone();
        self.restyle = Some(Arc::new(move |props, entry| {
            let params = typed::<P>(&schema.with_defaults(&entry.params))?;
            Ok(f(props, &params, &entry.node_id))
        }));
        self
    }

    pub fn retime<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(ClipValue, &P, Retiming<'_>) -> ClipValue + Send + Sync + 'static,
    {
        let schema = self.params.clone();
        self.retime = Some(Arc::new(move |clip, entry, fps| {
            let params = typed::<P>(&schema.with_defaults(&entry.params))?;
            Ok(f(
                clip,
                &params,
                Retiming {
                    node_id: &entry.node_id,
                    fps,
                },
            ))
        }));
        self
    }

    pub fn neutral<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&P) -> bool + Send + Sync + 'static,
    {
        let schema = self.params.clone();
        self.neutral = Some(Arc::new(move |values| {
            typed::<P>(&schema.with_defaults(values)).is_ok_and(|p| f(&p))
        }));
        self
    }

    pub fn is_neutral(&self, values: &Values) -> Option<bool> {
        self.neutral.as_ref().map(|f| f(values))
    }

    pub fn typed_params<P: DeserializeOwned>(&self, values: &Values) -> Result<P, String> {
        typed(&self.params.with_defaults(values))
    }

    pub fn apply_restyle(
        &self,
        props: RecordingProps,
        entry: &StyleEntry,
    ) -> Result<RecordingProps, String> {
        match &self.restyle {
            Some(f) => f(props, entry),
            None => Ok(props),
        }
    }

    pub fn apply_retime(
        &self,
        clip: ClipValue,
        entry: &StyleEntry,
        fps: f64,
    ) -> Result<ClipValue, String> {
        match &self.retime {
            Some(f) => f(clip, entry, fps),
            None => Ok(clip),
        }
    }

    pub fn has_compile(&self) -> bool {
        self.compile.is_some()
    }

    pub fn has_card(&self) -> bool {
        self.card.is_some()
    }

    pub fn input(&self, id: &str) -> Option<&PortSpec> {
        self.inputs.iter().find(|(k, _)| *k == id).map(|(_, s)| s)
    }

    pub fn output(&self, id: &str) -> Option<&PortSpec> {
        self.outputs.iter().find(|(k, _)| *k == id).map(|(_, s)| s)
    }

    pub fn defaults(&self) -> Values {
        self.params.defaults()
    }

    pub fn parse(&self, values: &Values, written: Option<&Values>) -> Result<Values, String> {
        let parsed = self.params.parse(values, written)?;
        if let Some(check) = &self.check {
            check(&parsed)?;
        }
        Ok(parsed)
    }

    pub fn run_compile(&self, ctx: &Ctx<'_, ()>) -> Result<Option<Outputs>, String> {
        match &self.compile {
            Some(f) => f(ctx),
            None => Ok(None),
        }
    }

    pub fn all_rows(&self, ctx: &CardCtx<'_, ()>) -> Vec<CardRow> {
        match &self.card {
            Some(f) => f(ctx).into_iter().flatten().collect(),
            None => Vec::new(),
        }
    }

    pub fn rows(&self, ctx: &CardCtx<'_, ()>) -> Vec<CardRow> {
        self.all_rows(ctx).into_iter().take(MAX_CARD_ROWS).collect()
    }

    pub fn card_for(
        &self,
        params: &Values,
        vars: &Vars,
        settings: &WorkflowSettings,
        result: Option<&NodeResult>,
        inputs: &CardInputs,
    ) -> Vec<CardRow> {
        let values = self.params.card_values(params, vars);
        self.rows(&CardCtx {
            params: (),
            values: &values,
            settings,
            result,
            inputs,
        })
    }

    pub fn mutation(&self, values: &Values) -> Option<String> {
        self.mutates.as_ref().and_then(|f| f(values))
    }
}

pub fn row(icon: Icon, text: impl Into<String>) -> CardRow {
    CardRow {
        icon,
        text: text.into(),
        title: None,
        tone: None,
        swatches: None,
    }
}

pub fn missing(icon: Icon, text: impl Into<String>) -> CardRow {
    row(icon, text).tone(Tone::Warn)
}

pub trait RowExt: Sized {
    fn tone(self, tone: Tone) -> Self;
    fn titled(self, title: impl Into<String>) -> Self;
    fn muted_if(self, when: bool) -> Self;
    fn warn_if(self, when: bool) -> Self;
    fn swatches(self, colors: Vec<String>) -> Self;
}

impl RowExt for CardRow {
    fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    fn titled(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    fn muted_if(self, when: bool) -> Self {
        if when { self.tone(Tone::Muted) } else { self }
    }

    fn warn_if(self, when: bool) -> Self {
        if when { self.tone(Tone::Warn) } else { self }
    }

    fn swatches(mut self, colors: Vec<String>) -> Self {
        self.swatches = Some(colors);
        self
    }
}

pub fn outputs(entries: impl IntoIterator<Item = (&'static str, CompileValue)>) -> Outputs {
    entries
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
}

fn append_entry<P>(
    ctx: &Ctx<'_, P>,
    port: &'static str,
    existing: Option<&EntriesValue>,
    wrap: fn(EntriesValue) -> CompileValue,
) -> Outputs {
    let mut entries = existing.map(|e| e.entries.clone()).unwrap_or_default();
    entries.push(ctx.entry());
    outputs([(port, wrap(EntriesValue { entries }))])
}

impl NodeDef {
    pub fn style_modifier<P: Params>(
        kind: impl Into<Cow<'static, str>>,
        title: &str,
        category: Category,
    ) -> Self {
        Self::of::<P>(kind, title, category, Phase::Compile)
            .inputs(vec![(STYLE_PORT, opt(PortType::RecordingStyle, "Style"))])
            .outputs(vec![(STYLE_PORT, port(PortType::RecordingStyle, "Style"))])
            .compiles(|ctx: &Ctx<'_, P>| {
                append_entry(
                    ctx,
                    STYLE_PORT,
                    ctx.input(STYLE_PORT).and_then(CompileValue::as_style),
                    CompileValue::RecordingStyle,
                )
            })
    }

    pub fn grade_modifier<P: Params>(
        kind: impl Into<Cow<'static, str>>,
        title: &str,
        category: Category,
    ) -> Self {
        Self::of::<P>(kind, title, category, Phase::Compile)
            .inputs(vec![(GRADE_PORT, opt(PortType::Grade, "Grade"))])
            .outputs(vec![(GRADE_PORT, port(PortType::Grade, "Grade"))])
            .compiles(|ctx: &Ctx<'_, P>| {
                append_entry(
                    ctx,
                    GRADE_PORT,
                    ctx.input(GRADE_PORT).and_then(CompileValue::as_grade),
                    CompileValue::Grade,
                )
            })
    }

    pub fn recording_modifier<P: Params>(
        kind: impl Into<Cow<'static, str>>,
        title: &str,
        category: Category,
    ) -> Self {
        Self::of::<P>(kind, title, category, Phase::Compile)
            .inputs(vec![(
                RECORDING_PORT,
                port(PortType::Recording, "Recording"),
            )])
            .outputs(vec![(
                RECORDING_PORT,
                port(PortType::Recording, "Recording"),
            )])
            .compiles(|ctx: &Ctx<'_, P>| {
                match ctx
                    .input(RECORDING_PORT)
                    .and_then(CompileValue::as_recording)
                {
                    Some(recording) => {
                        let mut recording = recording.clone();
                        recording.style.push(ctx.entry());
                        outputs([(RECORDING_PORT, CompileValue::Recording(recording))])
                    }
                    None => Outputs::new(),
                }
            })
    }

    pub fn clip_modifier<P: Params>(
        kind: impl Into<Cow<'static, str>>,
        title: &str,
        category: Category,
    ) -> Self {
        Self::of::<P>(kind, title, category, Phase::Compile)
            .inputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
            .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    }
}

pub fn with_clip<P>(ctx: &Ctx<'_, P>, f: impl FnOnce(&ClipValue) -> ClipValue) -> Outputs {
    match ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) {
        Some(clip) => outputs([(CLIP_PORT, CompileValue::Clip(f(clip)))]),
        None => Outputs::new(),
    }
}
