use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use opsis_compile::asset_ref::project_ref;
use opsis_compile::compile::{CompileResult, compile_workflow, gather_inputs, parse_node_params};
use opsis_compile::errors::SettingsError;
use opsis_compile::node::NodeDef;
use opsis_compile::params::Values;
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_engine::tools::EngineFiles;
use opsis_project::paths::ref_to_path;
use opsis_types::events::{LogLevel, RunEvent};
use opsis_types::settings::AppSettings;
use opsis_types::values::{Artifact, Measured, NodeResult, PortValues};
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde::de::DeserializeOwned;
use tokio_util::sync::CancellationToken;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;
pub type Emit = Arc<dyn Fn(RunEvent) + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("{message}")]
    Failed {
        message: String,
        trace_id: Option<String>,
    },
    #[error("Cancelled")]
    Cancelled,
    #[error("{message}")]
    Tool { message: String, output: String },
    #[error(transparent)]
    Settings(#[from] SettingsError),
}

impl ExecError {
    pub fn new(message: impl Into<String>) -> Self {
        Self::Failed {
            message: message.into(),
            trace_id: None,
        }
    }

    pub fn traced(message: impl Into<String>, trace_id: Option<String>) -> Self {
        Self::Failed {
            message: message.into(),
            trace_id,
        }
    }

    pub fn trace_id(&self) -> Option<&str> {
        match self {
            Self::Failed { trace_id, .. } => trace_id.as_deref(),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ExecError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug, Default)]
pub struct ExecutorOutput {
    pub outputs: PortValues,
    pub artifacts: Vec<Artifact>,
    pub trace_id: Option<String>,
    pub measured: Vec<Measured>,
}

#[derive(Debug, Clone)]
pub struct RunEnv {
    pub app_version: String,
    pub engine: EngineFiles,
    pub public_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub settings: AppSettings,
}

type RunFn = Arc<
    dyn Fn(Arc<ExecutorContext>, PortValues) -> BoxFuture<Result<ExecutorOutput, ExecError>>
        + Send
        + Sync,
>;

pub type Stamps = BTreeMap<String, String>;

type StampFn = Arc<dyn Fn(&Path, &Values) -> Stamps + Send + Sync>;

#[derive(Clone)]
pub struct Executor {
    run: RunFn,
    stamp: Option<StampFn>,
}

impl Executor {
    pub fn stamped(
        mut self,
        stamp: impl Fn(&Path, &Values) -> Stamps + Send + Sync + 'static,
    ) -> Self {
        self.stamp = Some(Arc::new(stamp));
        self
    }

    pub fn run(
        &self,
        ctx: Arc<ExecutorContext>,
        inputs: PortValues,
    ) -> BoxFuture<Result<ExecutorOutput, ExecError>> {
        (self.run)(ctx, inputs)
    }

    pub fn is_stamped(&self) -> bool {
        self.stamp.is_some()
    }

    pub fn stamps(&self, project_dir: &Path, params: &Values) -> Stamps {
        self.stamp
            .as_ref()
            .map(|stamp| stamp(project_dir, params))
            .unwrap_or_default()
    }
}

pub fn executor<F, Fut>(run: F) -> Executor
where
    F: Fn(Arc<ExecutorContext>, PortValues) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ExecutorOutput, ExecError>> + Send + 'static,
{
    Executor {
        run: Arc::new(move |ctx, inputs| Box::pin(run(ctx, inputs))),
        stamp: None,
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Two executors for {0}")]
pub struct TwoExecutors(pub String);

#[derive(Default, Clone)]
pub struct Executors {
    by_type: HashMap<String, Executor>,
}

impl Executors {
    pub fn add(&mut self, kind: &str, run: Executor) -> Result<(), TwoExecutors> {
        if self.by_type.insert(kind.to_owned(), run).is_some() {
            return Err(TwoExecutors(kind.to_owned()));
        }
        Ok(())
    }

    pub fn extend(&mut self, list: Vec<(&str, Executor)>) -> Result<(), TwoExecutors> {
        list.into_iter()
            .try_for_each(|(kind, run)| self.add(kind, run))
    }

    pub fn get(&self, kind: &str) -> Option<&Executor> {
        self.by_type.get(kind)
    }

    pub fn kinds(&self) -> impl Iterator<Item = &str> {
        self.by_type.keys().map(String::as_str)
    }
}

struct Stored {
    results: IndexMap<String, NodeResult>,
    version: u64,
    compiled: Option<(u64, Arc<CompileResult>)>,
}

pub struct ResultStore {
    workflow: Arc<Workflow>,
    stored: Mutex<Stored>,
}

impl ResultStore {
    pub fn new(workflow: Arc<Workflow>) -> Self {
        Self {
            workflow,
            stored: Mutex::new(Stored {
                results: IndexMap::new(),
                version: 0,
                compiled: None,
            }),
        }
    }

    pub fn set(&self, id: &str, result: NodeResult) {
        let mut stored = self.stored.lock().expect("results");
        stored.results.insert(id.to_owned(), result);
        stored.version += 1;
    }

    pub fn results(&self) -> IndexMap<String, NodeResult> {
        self.stored.lock().expect("results").results.clone()
    }

    pub fn compile(&self) -> Arc<CompileResult> {
        let mut stored = self.stored.lock().expect("results");
        if let Some((version, compiled)) = &stored.compiled
            && *version == stored.version
        {
            return Arc::clone(compiled);
        }
        let compiled = Arc::new(compile_workflow(&self.workflow, Some(&stored.results)));
        stored.compiled = Some((stored.version, Arc::clone(&compiled)));
        compiled
    }

    pub fn inputs_for(&self, node: &WorkflowNode, def: &NodeDef) -> PortValues {
        let compiled = self.compile();
        let results = self.results();
        gather_inputs(&self.workflow, def, &node.id, |e| {
            results
                .get(&e.source)
                .and_then(|r| r.outputs.get(&e.source_handle))
                .or_else(|| {
                    compiled
                        .values
                        .get(&e.source)
                        .and_then(|v| v.get(&e.source_handle))
                })
        })
    }
}

pub struct ExecutorContext {
    pub run_id: String,
    pub node: WorkflowNode,
    pub workflow: Arc<Workflow>,
    pub env: Arc<RunEnv>,
    pub project_dir: PathBuf,
    pub dir: PathBuf,
    pub rel_dir: String,
    pub out_dir: PathBuf,
    pub hash: String,
    pub forced: bool,
    pub vars: Arc<Vars>,
    pub cancel: CancellationToken,
    pub emit: Emit,
    pub store: Arc<ResultStore>,
}

pub fn now_ms() -> i64 {
    jiff::Timestamp::now().as_millisecond()
}

impl ExecutorContext {
    pub fn log(&self, level: LogLevel, message: impl Into<String>) {
        (self.emit)(RunEvent::NodeLog {
            run_id: self.run_id.clone(),
            node_id: Some(self.node.id.clone()),
            level,
            message: message.into(),
            at: now_ms(),
        });
    }

    pub fn progress(&self, progress: f64, message: Option<&str>) {
        (self.emit)(RunEvent::NodeProgress {
            run_id: self.run_id.clone(),
            node_id: self.node.id.clone(),
            progress,
            message: message.map(str::to_owned),
        });
    }

    pub fn compile(&self) -> Arc<CompileResult> {
        self.store.compile()
    }

    pub fn results(&self) -> IndexMap<String, NodeResult> {
        self.store.results()
    }

    pub fn params<P: DeserializeOwned>(&self) -> Result<P, ExecError> {
        let def = node_def(&self.node.kind)
            .ok_or_else(|| ExecError::new("This kind of node can't run."))?;
        let values = parse_node_params(def, &self.node, &self.vars)?;
        Ok(def.typed_params(&values)?)
    }

    pub fn rel(&self, file: &str) -> String {
        format!("{}/{file}", self.rel_dir)
    }

    pub fn project_file_ref(&self, file: &str) -> String {
        project_ref(&self.rel(file))
    }

    pub fn ref_to_path(&self, reference: &str) -> Result<PathBuf, ExecError> {
        ref_to_path(&self.project_dir, &self.env.public_dir, reference)
            .map_err(|e| ExecError::new(e.to_string()))
    }
}
