use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use futures::FutureExt;
use futures::future::{Shared, join_all};
use indexmap::IndexMap;
use opsis_compile::check::workflow_problems;
use opsis_compile::compile::parse_node_params;
use opsis_compile::graph::{
    IssueLevel, downstream_of, is_structural, run_targets, topo_sort, upstream_of,
};
use opsis_compile::node::{Category, NodeDef, Phase};
use opsis_compile::registry::{is_output_type, node_def, node_name};
use opsis_compile::vars::{Vars, workflow_vars};
use opsis_project::last_run::{read_last_run, write_last_run};
use opsis_project::paths::project_paths;
use opsis_project::paths::to_posix_rel;
use opsis_types::events::{CANCELLED, LogLevel, RunEvent, RunRequest};
use opsis_types::values::{LastRun, NodeFailure, NodeResult};
use opsis_types::workflow::{Workflow, WorkflowNode};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::cache::{read_cached, stamp_artifacts, write_cached};
use crate::context::{
    BoxFuture, Emit, ExecError, ExecutorContext, Executors, ResultStore, RunEnv, now_ms,
};
use crate::keys::{KeyInputs, hash_nodes};
use crate::progress::failure_message;

const SKIPPED_REASON: &str = "Something before it failed";
const CANT_RUN: &str = "This kind of node can't run.";

fn lane_limit(phase: Phase) -> usize {
    match phase {
        Phase::Capture => 3,
        Phase::Media => 4,
        Phase::Compile => 8,
        Phase::Render => 1,
        Phase::Post => 2,
    }
}

pub struct RunDeps {
    pub env: Arc<RunEnv>,
    pub executors: Arc<Executors>,
    pub emit: Emit,
    pub cancel: CancellationToken,
}

struct Job {
    node: WorkflowNode,
    def: &'static NodeDef,
    needs: Vec<String>,
}

struct RunState {
    run_id: String,
    workflow: Arc<Workflow>,
    vars: Arc<Vars>,
    project_dir: PathBuf,
    cache_dir: PathBuf,
    out_dir: PathBuf,
    deps: RunDeps,
    store: Arc<ResultStore>,
    forced: HashSet<String>,
    order: Vec<String>,
    hashes: Mutex<HashMap<String, String>>,
    failed: Mutex<Vec<String>>,
    skipped: Mutex<Vec<String>>,
    failures: Mutex<IndexMap<String, NodeFailure>>,
    outputs: Mutex<Vec<String>>,
    mutations: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

impl RunState {
    fn emit(&self, event: RunEvent) {
        (self.deps.emit)(event);
    }

    fn log(&self, level: LogLevel, message: String, node_id: Option<&str>) {
        self.emit(RunEvent::NodeLog {
            run_id: self.run_id.clone(),
            node_id: node_id.map(str::to_owned),
            level,
            message,
            at: now_ms(),
        });
    }

    fn done(&self, node_id: &str, cached: bool, result: Option<NodeResult>) {
        self.emit(RunEvent::NodeDone {
            run_id: self.run_id.clone(),
            node_id: node_id.to_owned(),
            cached,
            result: result.map(Box::new),
        });
    }

    fn hash_of(&self, id: &str) -> String {
        self.hashes
            .lock()
            .expect("hashes")
            .get(id)
            .cloned()
            .unwrap_or_default()
    }

    fn rekey(&self) {
        let nodes: Vec<&WorkflowNode> = self
            .order
            .iter()
            .filter_map(|id| self.workflow.nodes.iter().find(|n| &n.id == id))
            .collect();
        let fresh = hash_nodes(&nodes, &self.key_inputs());
        *self.hashes.lock().expect("hashes") = fresh;
    }

    fn key_inputs(&self) -> KeyInputs<'_> {
        KeyInputs {
            workflow: &self.workflow,
            vars: &self.vars,
            project_dir: &self.project_dir,
            app_version: &self.deps.env.app_version,
        }
    }

    fn mutation_lock(&self, key: &str) -> Arc<tokio::sync::Mutex<()>> {
        Arc::clone(
            self.mutations
                .lock()
                .expect("mutations")
                .entry(key.to_owned())
                .or_default(),
        )
    }

    fn blocked(&self, needs: &[String]) -> bool {
        let failed = self.failed.lock().expect("failed");
        let skipped = self.skipped.lock().expect("skipped");
        needs
            .iter()
            .any(|d| failed.contains(d) || skipped.contains(d))
    }

    fn collect_outputs(&self, node: &WorkflowNode, result: &NodeResult) {
        if is_output_type(&node.kind) {
            self.outputs
                .lock()
                .expect("outputs")
                .extend(result.artifacts.iter().map(|a| a.path.clone()));
        }
    }

    fn mark_chain(&self, node: &WorkflowNode, def: &NodeDef) {
        if def.category != Category::Capture {
            return;
        }
        for up in upstream_of(&self.workflow, &node.id) {
            let is_virtual = self
                .workflow
                .nodes
                .iter()
                .find(|n| n.id == up)
                .and_then(|n| node_def(&n.kind))
                .is_some_and(|d| d.is_virtual);
            if is_virtual {
                self.done(&up, true, None);
            }
        }
    }

    fn reuse_cached(&self, job: &Job) -> bool {
        let hash = self.hash_of(&job.node.id);
        let cached = match read_cached(&self.cache_dir, &hash, &self.project_dir) {
            Ok(Some(cached)) => cached,
            Ok(None) => return false,
            Err(error) => {
                log::warn!("couldn't read the saved result of {}: {error}", job.node.id);
                return false;
            }
        };
        self.store.set(&job.node.id, cached.clone());
        self.mark_chain(&job.node, job.def);
        self.collect_outputs(&job.node, &cached);
        self.done(&job.node.id, true, Some(cached));
        true
    }

    fn fail(&self, id: &str, error: &ExecError) {
        self.failed.lock().expect("failed").push(id.to_owned());
        if let ExecError::Tool { output, .. } = error {
            log::warn!("{id}: {output}");
        }
        let cancelled = self.deps.cancel.is_cancelled();
        let message = if cancelled {
            CANCELLED.to_owned()
        } else {
            error.to_string()
        };
        let trace_id = error.trace_id().map(str::to_owned);
        self.emit(RunEvent::NodeError {
            run_id: self.run_id.clone(),
            node_id: id.to_owned(),
            message: message.clone(),
            stack: None,
            trace_id: trace_id.clone(),
        });
        if !cancelled {
            self.failures.lock().expect("failures").insert(
                id.to_owned(),
                NodeFailure {
                    message,
                    at: now_ms(),
                    trace_id,
                },
            );
        }
    }

    async fn run_node(self: Arc<Self>, job: Arc<Job>) {
        if self.deps.cancel.is_cancelled() {
            return;
        }
        if self.blocked(&job.needs) {
            self.skipped
                .lock()
                .expect("skipped")
                .push(job.node.id.clone());
            self.emit(RunEvent::NodeSkipped {
                run_id: self.run_id.clone(),
                node_id: job.node.id.clone(),
                reason: SKIPPED_REASON.to_owned(),
            });
            return;
        }
        if !self.forced.contains(&job.node.id) && self.reuse_cached(&job) {
            return;
        }
        match Arc::clone(&self).execute(&job).await {
            Ok(result) => {
                if let Err(error) = write_cached(&self.cache_dir, &result) {
                    self.log(
                        LogLevel::Warn,
                        format!("Couldn't save the result: {error}"),
                        Some(&job.node.id),
                    );
                }
                self.store.set(&job.node.id, result.clone());
                self.collect_outputs(&job.node, &result);
                self.done(&job.node.id, false, Some(result));
            }
            Err(error) => self.fail(&job.node.id, &error),
        }
    }

    async fn execute(self: Arc<Self>, job: &Job) -> Result<NodeResult, ExecError> {
        let run = self
            .deps
            .executors
            .get(&job.node.kind)
            .cloned()
            .ok_or_else(|| ExecError::new(CANT_RUN))?;
        let started = now_ms();
        self.emit(RunEvent::NodeStart {
            run_id: self.run_id.clone(),
            node_id: job.node.id.clone(),
            at: started,
        });
        let hash = self.hash_of(&job.node.id);
        let dir = self.cache_dir.join(&hash);
        std::fs::create_dir_all(&dir)?;
        let ctx = Arc::new(ExecutorContext {
            run_id: self.run_id.clone(),
            node: job.node.clone(),
            workflow: Arc::clone(&self.workflow),
            env: Arc::clone(&self.deps.env),
            project_dir: self.project_dir.clone(),
            rel_dir: to_posix_rel(&self.project_dir, &dir).unwrap_or_default(),
            dir,
            out_dir: self.out_dir.clone(),
            hash: hash.clone(),
            forced: self.forced.contains(&job.node.id),
            vars: Arc::clone(&self.vars),
            cancel: self.deps.cancel.clone(),
            emit: Arc::clone(&self.deps.emit),
            store: Arc::clone(&self.store),
        });
        let inputs = self.store.inputs_for(&job.node, job.def);
        let out = run(ctx, inputs).await?;
        let mut artifacts = out.artifacts;
        for artifact in &mut artifacts {
            match std::fs::metadata(self.project_dir.join(&artifact.path)) {
                Ok(meta) => artifact.bytes = Some(meta.len() as f64),
                Err(error) => self.log(
                    LogLevel::Warn,
                    format!("Couldn't read {}: {error}", artifact.name()),
                    Some(&job.node.id),
                ),
            }
        }
        let finished = now_ms();
        let mut result = NodeResult {
            hash,
            outputs: out.outputs,
            artifacts,
            finished_at: finished,
            duration_ms: finished - started,
            trace_id: out.trace_id,
        };
        if !job.def.secret_params.is_empty() {
            self.rekey();
            let fresh = self.hash_of(&job.node.id);
            if fresh != result.hash {
                if let Err(error) = std::fs::remove_dir(self.cache_dir.join(&result.hash)) {
                    log::debug!("kept {}: {error}", result.hash);
                }
                result.hash = fresh;
            }
        }
        Ok(stamp_artifacts(result, &self.project_dir)?)
    }
}

fn needed_nodes(req: &RunRequest, wf: &Workflow) -> HashSet<String> {
    let targets = req
        .target_node_ids
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| run_targets(wf));
    let mut needed = HashSet::new();
    for id in targets {
        needed.extend(upstream_of(wf, &id));
        needed.insert(id);
    }
    needed
}

fn forced_nodes(req: &RunRequest, wf: &Workflow) -> HashSet<String> {
    let mut forced: HashSet<String> = req
        .target_node_ids
        .clone()
        .unwrap_or_default()
        .into_iter()
        .collect();
    if let Some(from) = &req.from_node_id {
        forced.insert(from.clone());
        forced.extend(downstream_of(wf, from));
    }
    forced
}

fn report_invalid(run_id: &str, wf: &Workflow, needed: &HashSet<String>, emit: &Emit) -> bool {
    let issues: Vec<_> = workflow_problems(wf)
        .into_iter()
        .filter(|i| {
            i.level == IssueLevel::Error && i.node_id.as_ref().is_none_or(|id| needed.contains(id))
        })
        .collect();
    let Some(first) = issues.first() else {
        return false;
    };
    for issue in &issues {
        match &issue.node_id {
            Some(id) => emit(RunEvent::NodeError {
                run_id: run_id.to_owned(),
                node_id: id.clone(),
                message: issue.message.clone(),
                stack: None,
                trace_id: None,
            }),
            None => emit(RunEvent::NodeLog {
                run_id: run_id.to_owned(),
                node_id: None,
                level: LogLevel::Error,
                message: issue.message.clone(),
                at: now_ms(),
            }),
        }
    }
    let mut failed: Vec<String> = Vec::new();
    for id in issues.iter().filter_map(|i| i.node_id.clone()) {
        if !failed.contains(&id) {
            failed.push(id);
        }
    }
    emit(run_failed(run_id, &first.message, false, failed));
    true
}

pub fn run_failed(
    run_id: &str,
    error: &str,
    cancelled: bool,
    failed_node_ids: Vec<String>,
) -> RunEvent {
    RunEvent::RunDone {
        run_id: run_id.to_owned(),
        ok: false,
        cancelled,
        error: Some(error.to_owned()),
        failed_node_ids: Some(failed_node_ids),
        skipped_node_ids: Some(Vec::new()),
        outputs: Vec::new(),
        at: now_ms(),
    }
}

fn merge_last_run(
    previous: LastRun,
    results: &IndexMap<String, NodeResult>,
    failures: &IndexMap<String, NodeFailure>,
    exists: &dyn Fn(&str) -> bool,
) -> LastRun {
    let mut nodes = previous.nodes;
    nodes.extend(results.clone());
    nodes.retain(|id, _| exists(id));
    let mut failed = previous.failures.unwrap_or_default();
    failed.extend(failures.clone());
    failed.retain(|id, _| !results.contains_key(id) && exists(id));
    LastRun {
        updated_at: now_ms(),
        nodes,
        failures: Some(failed),
    }
}

fn finish(state: &RunState, previous: LastRun) {
    let ids: HashSet<&str> = state.workflow.nodes.iter().map(|n| n.id.as_str()).collect();
    let merged = merge_last_run(
        previous,
        &state.store.results(),
        &state.failures.lock().expect("failures"),
        &|id| ids.contains(id),
    );
    if let Err(error) = write_last_run(&state.project_dir, &merged) {
        state.log(
            LogLevel::Warn,
            format!("Couldn't save the run results: {error}"),
            None,
        );
    }
    let failed = state.failed.lock().expect("failed").clone();
    let skipped = state.skipped.lock().expect("skipped").clone();
    let name_of = |id: &String| {
        state
            .workflow
            .nodes
            .iter()
            .find(|n| &n.id == id)
            .map_or_else(|| id.clone(), node_name)
    };
    let any_failed = !failed.is_empty() || !skipped.is_empty();
    let cancelled = state.deps.cancel.is_cancelled();
    let error = (!cancelled && any_failed).then(|| {
        failure_message(
            &failed.iter().map(name_of).collect::<Vec<_>>(),
            skipped.len(),
        )
    });
    state.emit(RunEvent::RunDone {
        run_id: state.run_id.clone(),
        ok: !cancelled && !any_failed,
        cancelled,
        error,
        failed_node_ids: Some(if cancelled { Vec::new() } else { failed }),
        skipped_node_ids: Some(if cancelled { Vec::new() } else { skipped }),
        outputs: state.outputs.lock().expect("outputs").clone(),
        at: now_ms(),
    });
}

pub async fn run_workflow(run_id: &str, req: RunRequest, deps: RunDeps) {
    let workflow = Arc::new(req.workflow.clone());
    let project_dir = PathBuf::from(&req.project_dir);
    let paths = project_paths(&project_dir);
    for dir in [&paths.cache, &paths.out] {
        if let Err(error) = std::fs::create_dir_all(dir) {
            (deps.emit)(run_failed(run_id, &error.to_string(), false, Vec::new()));
            return;
        }
    }
    let needed = needed_nodes(&req, &workflow);
    if report_invalid(run_id, &workflow, &needed, &deps.emit) {
        return;
    }
    let sorted = topo_sort(&workflow);
    let order: Vec<String> = sorted.sorted.iter().map(|n| n.id.clone()).collect();
    let to_run: Vec<(WorkflowNode, &'static NodeDef)> = sorted
        .sorted
        .iter()
        .filter(|n| needed.contains(&n.id) && !n.is_disabled() && !is_structural(n))
        .filter_map(|n| {
            node_def(&n.kind)
                .filter(|d| d.executes)
                .map(|d| ((*n).clone(), d))
        })
        .collect();
    (deps.emit)(RunEvent::RunStart {
        run_id: run_id.to_owned(),
        node_ids: to_run.iter().map(|(n, _)| n.id.clone()).collect(),
        at: now_ms(),
    });
    let vars = Arc::new(workflow_vars(&workflow.name, &workflow.variables));
    let previous = match read_last_run(&project_dir) {
        Ok(read) => read.unwrap_or(LastRun {
            updated_at: 0.0,
            nodes: IndexMap::new(),
            failures: None,
        }),
        Err(error) => {
            (deps.emit)(run_failed(run_id, &error.to_string(), false, Vec::new()));
            return;
        }
    };
    let state = Arc::new(RunState {
        run_id: run_id.to_owned(),
        store: Arc::new(ResultStore::new(Arc::clone(&workflow))),
        workflow: Arc::clone(&workflow),
        vars,
        project_dir: project_dir.clone(),
        cache_dir: paths.cache.clone(),
        out_dir: paths.out.clone(),
        forced: forced_nodes(&req, &workflow),
        order,
        hashes: Mutex::new(HashMap::new()),
        failed: Mutex::default(),
        skipped: Mutex::default(),
        failures: Mutex::default(),
        outputs: Mutex::default(),
        mutations: Mutex::default(),
        deps,
    });
    state.rekey();
    for (id, result) in &previous.nodes {
        if state.hash_of(id) == result.hash {
            state.store.set(id, result.clone());
        }
    }
    let run_set: HashSet<String> = to_run.iter().map(|(n, _)| n.id.clone()).collect();
    let lanes: HashMap<Phase, Arc<Semaphore>> = [
        Phase::Capture,
        Phase::Media,
        Phase::Compile,
        Phase::Render,
        Phase::Post,
    ]
    .into_iter()
    .map(|p| (p, Arc::new(Semaphore::new(lane_limit(p)))))
    .collect();
    let mut futures: IndexMap<String, Shared<BoxFuture<()>>> = IndexMap::new();
    for (node, def) in to_run {
        let needs: Vec<String> = upstream_of(&workflow, &node.id)
            .into_iter()
            .filter(|u| *u != node.id && run_set.contains(u))
            .collect();
        let waits: Vec<Shared<BoxFuture<()>>> = needs
            .iter()
            .filter_map(|d| futures.get(d).cloned())
            .collect();
        let mutation = match parse_node_params(def, &node, &state.vars) {
            Ok(values) => def.mutation(&values),
            Err(problem) => {
                log::warn!("{} has settings that don't parse: {problem}", node.id);
                None
            }
        };
        let lane = Arc::clone(&lanes[&def.phase]);
        let job = Arc::new(Job {
            node: node.clone(),
            def,
            needs,
        });
        let st = Arc::clone(&state);
        let future: BoxFuture<()> = Box::pin(async move {
            join_all(waits).await;
            let lock = mutation.map(|key| st.mutation_lock(&key));
            let _held = match &lock {
                Some(lock) => Some(lock.lock().await),
                None => None,
            };
            let Ok(_permit) = lane.acquire_owned().await else {
                return;
            };
            st.run_node(job).await;
        });
        futures.insert(node.id.clone(), future.shared());
    }
    join_all(futures.into_values()).await;
    finish(&state, previous);
}
