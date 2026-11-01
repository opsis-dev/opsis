use std::borrow::Cow;
use std::sync::Arc;

use indexmap::IndexMap;
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::recording::RecordingProps;
use opsis_types::values::{
    BrowserAction, ClipValue, CompileValue, EntriesValue, Hits, MediaKind, MediaValue, NodeResult,
    PortValues, StyleEntry,
};
use opsis_types::workflow::{PortType, Workflow, WorkflowSettings};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::elements::ElementMeta;
use crate::errors::{CompileError, SettingsError};
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
    Image,
    Output,
    Utility,
}

impl Category {
    pub const ALL: [Self; 16] = [
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
        Self::Image,
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
            Self::Image => "Image",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileNeed {
    Image,
    Picture,
    Moving,
    Sound,
    SoundFile,
}

impl FileNeed {
    pub fn refuses(self, media: &MediaValue) -> bool {
        match self {
            Self::Image => media.kind != MediaKind::Image,
            Self::Picture => media.kind == MediaKind::Audio,
            Self::Moving => media.kind == MediaKind::Image,
            Self::Sound => media.kind == MediaKind::Image || media.has_audio == Some(false),
            Self::SoundFile => media.kind != MediaKind::Audio,
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Image => "The connected file isn't an image.",
            Self::Picture => "The connected file is only sound, with no picture.",
            Self::Moving => "The connected file is a still image, with nothing that plays.",
            Self::Sound => "The connected file has no sound.",
            Self::SoundFile => {
                "This works on sound files, so a video doesn't lose its sync. Take the sound out with Extract sound first."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSpec {
    pub kind: PortType,
    pub label: Option<Cow<'static, str>>,
    pub optional: bool,
    pub many: bool,
    pub need: Option<FileNeed>,
}

impl PortSpec {
    pub fn label(&self) -> String {
        self.label
            .as_deref()
            .map_or_else(|| crate::text::humanize(self.kind.word()), str::to_owned)
    }

    #[must_use]
    pub fn optional(self) -> Self {
        Self {
            optional: true,
            ..self
        }
    }

    #[must_use]
    pub fn many(self) -> Self {
        Self {
            optional: true,
            many: true,
            ..self
        }
    }

    fn needing(&mut self, need: FileNeed) {
        if self.kind == PortType::Media {
            self.need = Some(need);
        }
    }
}

pub fn port(kind: PortType, label: impl Into<Cow<'static, str>>) -> PortSpec {
    PortSpec {
        kind,
        label: Some(label.into()),
        optional: false,
        many: false,
        need: None,
    }
}

pub fn opt(kind: PortType, label: impl Into<Cow<'static, str>>) -> PortSpec {
    port(kind, label).optional()
}

pub fn many(kind: PortType, label: impl Into<Cow<'static, str>>) -> PortSpec {
    port(kind, label).many()
}

pub type Ports = Vec<(&'static str, PortSpec)>;

pub type FamilyLabel = fn(usize, Option<usize>) -> String;

pub fn numbered_index(prefix: &str, handle: &str) -> Option<usize> {
    let digits = handle.strip_prefix(prefix)?;
    if digits.is_empty() || digits.starts_with('0') || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<usize>().ok()?.checked_sub(1)
}

#[derive(Debug, Clone)]
pub struct PortFamily {
    pub prefix: &'static str,
    pub required: usize,
    pub shown: usize,
    pub refs: &'static [(&'static str, &'static str)],
    label: FamilyLabel,
    required_spec: PortSpec,
    optional_spec: PortSpec,
}

impl PortFamily {
    pub fn new(prefix: &'static str, kind: PortType, required: usize, label: FamilyLabel) -> Self {
        let spec = PortSpec {
            kind,
            label: None,
            optional: false,
            many: false,
            need: None,
        };
        Self {
            prefix,
            required,
            shown: required.max(2),
            refs: &[],
            label,
            optional_spec: spec.clone().optional(),
            required_spec: spec,
        }
    }

    #[must_use]
    pub fn with_refs(mut self, refs: &'static [(&'static str, &'static str)]) -> Self {
        self.refs = refs;
        self
    }

    pub fn kind(&self) -> PortType {
        self.required_spec.kind
    }

    pub fn index(&self, handle: &str) -> Option<usize> {
        numbered_index(self.prefix, handle)
    }

    pub fn port(&self, index: usize) -> String {
        format!("{}{}", self.prefix, index + 1)
    }

    pub fn spec(&self, index: usize) -> &PortSpec {
        if index < self.required {
            &self.required_spec
        } else {
            &self.optional_spec
        }
    }

    pub fn label(&self, index: usize, last: Option<usize>) -> String {
        (self.label)(index, last)
    }

    pub fn connected<'a>(&self, handles: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
        let mut found: Vec<usize> = handles.into_iter().filter_map(|h| self.index(h)).collect();
        found.sort_unstable();
        found.dedup();
        found
    }

    pub fn shown_count(&self, connected: &[usize]) -> usize {
        connected.last().map_or(0, |last| last + 2).max(self.shown)
    }
}
pub type Inputs = PortValues;
pub type Outputs = PortValues;

pub const CLIP_PORT: &str = "clip";
pub const RECORDING_PORT: &str = "recording";
pub const STYLE_PORT: &str = "style";
pub const GRADE_PORT: &str = "grade";
pub const SESSION_PORT: &str = "session";

#[derive(Debug)]
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
    pub clip_frames: Option<u32>,
    pub frames: IndexMap<String, u32>,
    pub items: IndexMap<String, usize>,
    pub takes: IndexMap<String, Option<InputTake>>,
    pub media: IndexMap<String, InputMedia>,
    pub pixels: IndexMap<String, (u32, u32)>,
    pub captions_ms: IndexMap<String, f64>,
    pub steps: IndexMap<String, Vec<BrowserAction>>,
    pub style: IndexMap<String, Vec<String>>,
    pub grade: IndexMap<String, Vec<StyleEntry>>,
    pub missing_files: Vec<String>,
    pub hits: Option<Hits>,
    pub names: IndexMap<String, String>,
}

impl CardInputs {
    pub fn is_connected(&self, port: &str) -> bool {
        self.connected.iter().any(|c| c == port)
    }
}

#[derive(Debug)]
pub struct CardCtx<'a, P> {
    pub params: P,
    pub values: &'a Values,
    pub settings: &'a WorkflowSettings,
    pub result: Option<&'a NodeResult>,
    pub inputs: &'a CardInputs,
}

impl<P> CardCtx<'_, P> {
    pub fn fps(&self) -> u32 {
        self.settings.fps
    }

    pub fn connected(&self, port: &str) -> bool {
        self.inputs.is_connected(port)
    }
}

pub type Rows = Vec<Option<CardRow>>;
pub const MAX_CARD_ROWS: usize = 4;

type CompileFn = Arc<dyn Fn(&Ctx<'_, ()>) -> Result<Option<Outputs>, CompileError> + Send + Sync>;
type CardFn = Arc<dyn Fn(&CardCtx<'_, ()>) -> Rows + Send + Sync>;
type MutatesFn = Arc<dyn Fn(&Values) -> Option<String> + Send + Sync>;
type ClaimsFn = Arc<dyn Fn(&Values) -> Vec<Claim> + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Claim {
    SignIn(String),
    File(String),
}

impl Claim {
    pub fn key(&self) -> String {
        match self {
            Self::SignIn(name) => {
                format!("sign-in:{}", opsis_types::secrets::secret_file_name(name))
            }
            Self::File(name) => format!("file:{name}"),
        }
    }
}
type HostDefaultsFn = fn(Platform) -> Values;
type CheckFn = Arc<dyn Fn(&Values) -> Result<(), SettingsError> + Send + Sync>;
type RestyleFn =
    Arc<dyn Fn(RecordingProps, &StyleEntry) -> Result<RecordingProps, SettingsError> + Send + Sync>;
type NeutralFn = Arc<dyn Fn(&Values) -> bool + Send + Sync>;
type RetimeFn =
    Arc<dyn Fn(ClipValue, &StyleEntry, u32) -> Result<ClipValue, SettingsError> + Send + Sync>;

#[derive(Debug, Clone, Copy)]
pub struct Retiming<'a> {
    pub node_id: &'a str,
    pub fps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Mac,
    Windows,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

#[derive(Clone)]
pub struct NodeDef {
    pub kind: Cow<'static, str>,
    pub title: String,
    pub category: Category,
    pub description: Option<String>,
    pub phase: Phase,
    pub inputs: Ports,
    pub family: Option<PortFamily>,
    pub outputs: Ports,
    pub params: ParamSchema,
    pub is_virtual: bool,
    pub action: bool,
    pub executes: bool,
    pub uses_project_size: bool,
    pub uses_project_fonts: bool,
    pub element: Option<String>,
    pub hidden: bool,
    pub look: Option<NodeLook>,
    pub card_height: Option<f64>,
    pub host_defaults: Option<HostDefaultsFn>,
    check: Option<CheckFn>,
    compile: Option<CompileFn>,
    card: Option<CardFn>,
    mutates: Option<MutatesFn>,
    claims: Option<ClaimsFn>,
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

fn typed<P: DeserializeOwned>(values: &Values) -> Result<P, SettingsError> {
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
            family: None,
            outputs: Vec::new(),
            params,
            is_virtual: false,
            action: false,
            executes: false,
            uses_project_size: false,
            uses_project_fonts: false,
            element: None,
            hidden: false,
            look: None,
            card_height: None,
            host_defaults: None,
            check: None,
            compile: None,
            card: None,
            mutates: None,
            claims: None,
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

    #[must_use]
    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn inputs(mut self, ports: Ports) -> Self {
        self.inputs = ports;
        self
    }

    #[must_use]
    pub fn family(mut self, family: PortFamily) -> Self {
        self.family = Some(family);
        self
    }

    #[must_use]
    pub fn needs(mut self, need: FileNeed) -> Self {
        for (_, spec) in &mut self.inputs {
            spec.needing(need);
        }
        if let Some(family) = &mut self.family {
            family.required_spec.needing(need);
            family.optional_spec.needing(need);
        }
        self
    }

    #[must_use]
    pub fn outputs(mut self, ports: Ports) -> Self {
        self.outputs = ports;
        self
    }

    #[must_use]
    pub fn virtual_node(mut self) -> Self {
        self.is_virtual = true;
        self
    }

    #[must_use]
    pub fn action(mut self) -> Self {
        self.action = true;
        self
    }

    #[must_use]
    pub fn look(mut self, look: NodeLook) -> Self {
        self.look = Some(look);
        self
    }

    #[must_use]
    pub fn executes(mut self) -> Self {
        self.executes = true;
        self
    }

    #[must_use]
    pub fn uses_project_size(mut self) -> Self {
        self.uses_project_size = true;
        self
    }

    #[must_use]
    pub fn uses_project_fonts(mut self) -> Self {
        self.uses_project_fonts = true;
        self
    }

    #[must_use]
    pub fn element(mut self, meta: &ElementMeta) -> Self {
        self.element = Some(meta.key.clone());
        self.look.clone_from(&meta.look);
        self
    }

    #[must_use]
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    #[must_use]
    pub fn card_height(mut self, height: f64) -> Self {
        self.card_height = Some(height);
        self
    }

    #[must_use]
    pub fn host_defaults(mut self, f: HostDefaultsFn) -> Self {
        self.host_defaults = Some(f);
        self
    }

    #[must_use]
    pub fn compile<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&Ctx<'_, P>) -> Result<Option<Outputs>, CompileError> + Send + Sync + 'static,
    {
        self.compile = Some(Arc::new(move |ctx| {
            f(&retyped(ctx, typed::<P>(ctx.values)?))
        }));
        self
    }

    #[must_use]
    pub fn compiles<P, F>(self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&Ctx<'_, P>) -> Outputs + Send + Sync + 'static,
    {
        self.compile(move |ctx: &Ctx<'_, P>| Ok(Some(f(ctx))))
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
    pub fn claims<P, F>(mut self, f: F) -> Self
    where
        P: DeserializeOwned + 'static,
        F: Fn(&P) -> Vec<Claim> + Send + Sync + 'static,
    {
        self.claims = Some(Arc::new(move |values| {
            typed::<P>(values).map_or_default(|p| f(&p))
        }));
        self
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
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

    pub fn typed_params<P: DeserializeOwned>(&self, values: &Values) -> Result<P, SettingsError> {
        typed(&self.params.with_defaults(values))
    }

    pub fn apply_restyle(
        &self,
        props: RecordingProps,
        entry: &StyleEntry,
    ) -> Result<RecordingProps, SettingsError> {
        match &self.restyle {
            Some(f) => f(props, entry),
            None => Ok(props),
        }
    }

    pub fn apply_retime(
        &self,
        clip: ClipValue,
        entry: &StyleEntry,
        fps: u32,
    ) -> Result<ClipValue, SettingsError> {
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
        self.inputs
            .iter()
            .find(|(k, _)| *k == id)
            .map(|(_, s)| s)
            .or_else(|| {
                let family = self.family.as_ref()?;
                family.index(id).map(|i| family.spec(i))
            })
    }

    pub fn family_index(&self, id: &str) -> Option<usize> {
        self.family.as_ref()?.index(id)
    }

    pub fn input_ports<'a>(
        &self,
        connected: impl IntoIterator<Item = &'a str>,
    ) -> Vec<(String, String, PortSpec)> {
        let mut ports: Vec<(String, String, PortSpec)> = self
            .inputs
            .iter()
            .map(|(id, spec)| ((*id).to_owned(), spec.label(), spec.clone()))
            .collect();
        if let Some(family) = &self.family {
            let found = family.connected(connected);
            let last = found.last().copied();
            ports.extend((0..family.shown_count(&found)).map(|i| {
                (
                    family.port(i),
                    family.label(i, last),
                    family.spec(i).clone(),
                )
            }));
        }
        ports
    }

    pub fn output(&self, id: &str) -> Option<&PortSpec> {
        self.outputs.iter().find(|(k, _)| *k == id).map(|(_, s)| s)
    }

    pub fn defaults(&self) -> Values {
        self.params.defaults()
    }

    pub fn parse(
        &self,
        values: &Values,
        written: Option<&Values>,
    ) -> Result<Values, SettingsError> {
        let parsed = self.params.parse(values, written)?;
        if let Some(check) = &self.check {
            check(&parsed)?;
        }
        Ok(parsed)
    }

    pub fn run_compile(&self, ctx: &Ctx<'_, ()>) -> Result<Option<Outputs>, CompileError> {
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

    pub fn claimed(&self, values: &Values) -> Vec<Claim> {
        self.claims
            .as_ref()
            .map_or_default(|f| f(&self.params.with_defaults(values)))
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
    #[must_use]
    fn tone(self, tone: Tone) -> Self;
    #[must_use]
    fn titled(self, title: impl Into<String>) -> Self;
    #[must_use]
    fn muted_if(self, when: bool) -> Self;
    #[must_use]
    fn warn_if(self, when: bool) -> Self;
    #[must_use]
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
    let mut entries = existing.map_or_default(|e| e.entries.clone());
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
