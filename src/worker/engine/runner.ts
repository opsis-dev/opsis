import { mkdir } from "node:fs/promises";
import path from "node:path";
import pLimit, { type LimitFunction } from "p-limit";
import { readLastRun, writeLastRun } from "../../engine/last-run";
import { logWarning } from "../../engine/log";
import { enginePaths, type EnginePaths } from "../../engine/paths";
import { PICTURE_VERSION } from "../../engine/picture-version";
import { projectPaths, toPosixRel } from "../../engine/project-paths";
import { workflowProblems } from "../../shared/api/check";
import { compileWorkflow, type CompileResult } from "../../shared/compile/compile";
import { CANCELLED, errorMessage } from "../../shared/errors";
import {
  downstreamOf,
  isStructural,
  loginDependencies,
  runTargets,
  topoSort,
  upstreamOf,
} from "../../shared/graph";
import type { RunRequest } from "../../shared/ipc";
import type { LiveMessage } from "../../shared/live";
import {
  interpolatedParams,
  parseNodeParams,
  type NodeDef,
  type NodePhase,
} from "../../shared/nodes/define";
import { getNodeDef, isOutputType, nodeName } from "../../shared/nodes/registry";
import { mapValues, pick } from "../../shared/objects";
import { failureMessage } from "../../shared/run-progress";
import { runFailed, type LogLevel, type RunEvent } from "../../shared/schema/events";
import type { LastRun, NodeFailure, NodeResult } from "../../shared/schema/values";
import type { Workflow, WorkflowNode } from "../../shared/schema/workflow";
import { secretName } from "../../shared/secrets";
import { interpolate, workflowVars } from "../../shared/vars";
import type { LoginRequest, WorkerEnv } from "../../shared/worker-protocol";
import type { LiveSink } from "../capture/take-run";
import { getExecutor } from "../executors";
import { createFfmpegTools, ToolError, type FfmpegTools } from "../media/ffmpeg";
import { nodeHash, readCached, writeCached } from "./cache";
import {
  TracedError,
  warnFailed,
  type Executor,
  type ExecutorContext,
  type ExecutorInputs,
  type NodeResults,
} from "./context";
import { fileStamps } from "./file-stamps";
import { decorateArtifacts } from "./posters";

type RunDeps = {
  env: WorkerEnv;
  emit: (event: RunEvent) => void;
  signal: AbortSignal;
  secrets: ExecutorContext["secrets"];
  login: (request: LoginRequest) => Promise<string>;
  live: ((message: LiveMessage) => void) | null;
};

type Log = (level: LogLevel, message: string, nodeId?: string) => void;

type RunContext = {
  runId: string;
  req: RunRequest;
  deps: RunDeps;
  vars: Record<string, string>;
  log: Log;
};

type Known = { node: WorkflowNode; def: NodeDef };

type Job = Known & { hash: string; needs: string[] };

const LANE_LIMITS: Readonly<Record<NodePhase, number>> = {
  capture: 3,
  media: 4,
  compile: 8,
  render: 1,
  post: 2,
};

const isCapture = (def: NodeDef | undefined) => def?.category === "Capture";

const doneEvent = (
  runId: string,
  nodeId: string,
  cached: boolean,
  result?: NodeResult,
): RunEvent => ({ type: "node:done", runId, nodeId, cached, result });

function knownNodes(workflow: Workflow): Map<string, Known> {
  const known = new Map<string, Known>();
  for (const node of workflow.nodes) {
    const def = getNodeDef(node.type);
    if (def) known.set(node.id, { node, def });
  }
  return known;
}

async function signInSaved(node: WorkflowNode, { vars, deps, log }: RunContext) {
  const name = secretName(interpolate(node.params, vars).secretName);
  const saved = await deps.secrets.read(name).catch((error: unknown) => {
    warnFailed(
      log,
      `Couldn't read the saved sign-in “${name}”, so it will be asked for again`,
      error,
    );
    return null;
  });
  return !!saved;
}

async function neededNodes(
  run: RunContext,
  known: Map<string, Known>,
  order: readonly WorkflowNode[],
): Promise<Set<string>> {
  const { workflow, targetNodeIds } = run.req;
  const targets = targetNodeIds?.length ? targetNodeIds : runTargets(workflow);
  const needed = new Set<string>();
  for (const id of targets) {
    needed.add(id);
    for (const up of upstreamOf(workflow, id)) needed.add(up);
  }
  const captures = order.filter((n) => needed.has(n.id) && isCapture(known.get(n.id)?.def));
  for (const { id } of captures) {
    for (const login of loginDependencies(workflow, id, run.vars)) {
      const node = known.get(login)?.node;
      if (needed.has(login) || !node) continue;
      if (!(await signInSaved(node, run))) needed.add(login);
    }
  }
  return needed;
}

function hashNodes(
  workflow: Workflow,
  order: readonly WorkflowNode[],
  vars: Record<string, string>,
  projectDir: string,
): Map<string, string> {
  const hashes = new Map<string, string>();
  for (const node of order) {
    const upstream = workflow.edges
      .filter((e) => e.target === node.id && hashes.has(e.source))
      .map((e): [string, string] => [e.targetHandle, `${hashes.get(e.source)}:${e.sourceHandle}`]);
    const def = getNodeDef(node.type);
    const params = interpolatedParams(def, node.params, vars);
    const files = fileStamps(projectDir, params);
    const keyed =
      def?.phase === "render"
        ? {
            params,
            settings: workflow.settings,
            mix: workflow.mix ?? null,
            picture: PICTURE_VERSION,
            vars,
            files,
          }
        : def?.usesProjectSize
          ? { params, files, size: pick(workflow.settings, ["width", "height", "fps"]) }
          : Object.keys(files).length
            ? { params, files }
            : params;
    hashes.set(node.id, nodeHash(node.type, keyed, upstream));
  }
  return hashes;
}

function requireHash(hashes: ReadonlyMap<string, string>, id: string): string {
  const hash = hashes.get(id);
  if (hash === undefined) throw new Error(`The node ${id} has no cache key.`);
  return hash;
}

function mergeLastRun(
  previous: LastRun,
  results: NodeResults,
  failures: Record<string, NodeFailure>,
  existing: (id: string) => boolean,
): LastRun {
  const nodes: Record<string, NodeResult> = {};
  for (const [id, r] of Object.entries({ ...previous.nodes, ...results }))
    if (r && existing(id)) nodes[id] = r;
  const failed: Record<string, NodeFailure> = {};
  for (const [id, f] of Object.entries({ ...previous.failures, ...failures }))
    if (!results[id] && existing(id)) failed[id] = f;
  return { updatedAt: Date.now(), nodes, failures: failed };
}

class Scheduler {
  private readonly lanes: Record<NodePhase, LimitFunction> = mapValues(LANE_LIMITS, (n) =>
    pLimit(n),
  );
  private readonly serial = new Map<string, Promise<unknown>>();
  private readonly promises = new Map<string, Promise<void>>();

  constructor(
    private readonly jobs: Map<string, Job>,
    private readonly mutationKey: (job: Job) => string | undefined,
    private readonly run: (job: Job) => Promise<void>,
  ) {}

  all(): Promise<void[]> {
    return Promise.all([...this.jobs.keys()].map((id) => this.schedule(id)));
  }

  private schedule(id: string): Promise<void> {
    const existing = this.promises.get(id);
    if (existing) return existing;
    const job = this.jobs.get(id);
    const p = job ? this.start(job) : Promise.resolve();
    this.promises.set(id, p);
    return p;
  }

  private async start(job: Job): Promise<void> {
    await Promise.all(job.needs.map((d) => this.schedule(d)));
    const lane = this.lanes[job.def.phase];
    const key = this.mutationKey(job);
    if (!key) return lane(() => this.run(job));
    const next = (this.serial.get(key) ?? Promise.resolve()).then(() => lane(() => this.run(job)));
    this.serial.set(
      key,
      next.catch(() => undefined),
    );
    return next;
  }
}

class ResultStore {
  readonly results: NodeResults = {};
  private version = 0;
  private compiled: { version: number; result: CompileResult } | null = null;

  constructor(private readonly workflow: Workflow) {}

  set(id: string, result: NodeResult) {
    this.results[id] = result;
    this.version++;
  }

  compile = (): CompileResult => {
    if (this.compiled?.version !== this.version)
      this.compiled = {
        version: this.version,
        result: compileWorkflow(this.workflow, { results: this.results }),
      };
    return this.compiled.result;
  };

  inputsFor(id: string): ExecutorInputs {
    const inputs: ExecutorInputs = {};
    const values: Partial<CompileResult["values"]> = this.compile().values;
    for (const e of this.workflow.edges) {
      if (e.target !== id) continue;
      const v =
        this.results[e.source]?.outputs[e.sourceHandle] ?? values[e.source]?.[e.sourceHandle];
      if (v) inputs[e.targetHandle] = v;
    }
    return inputs;
  }
}

function liveSink(run: RunContext, nodeId: string, title: string): LiveSink | null {
  const send = run.deps.live;
  if (!send) return null;
  const captureId = `${run.runId}:${nodeId}`;
  const minGap = 1000 / Math.max(1, run.deps.env.settings.live.fps);
  let lastFrame = 0;
  return {
    frame: (jpeg, width, height, t) => {
      if (t - lastFrame < minGap) return;
      lastFrame = t;
      send({ type: "frame", captureId, nodeId, jpeg: new Uint8Array(jpeg), width, height, t });
    },
    event: (event) =>
      send({
        type: "event",
        captureId,
        nodeId,
        t: Date.now(),
        event: event.kind === "start" ? { ...event, title } : event,
      }),
  };
}

type RunTools = { ffmpeg: FfmpegTools; engine: EnginePaths; outDir: string };

function executorContext(
  run: RunContext,
  tools: RunTools,
  store: ResultStore,
  { node, hash, dir }: { node: WorkflowNode; hash: string; dir: string },
): ExecutorContext {
  const { runId, req, deps, vars, log } = run;
  const { emit } = deps;
  const id = node.id;
  return {
    runId,
    node,
    workflow: req.workflow,
    env: deps.env,
    projectDir: req.projectDir,
    dir,
    relDir: toPosixRel(req.projectDir, dir),
    outDir: tools.outDir,
    hash,
    vars,
    signal: deps.signal,
    ffmpeg: tools.ffmpeg,
    engine: tools.engine,
    live: (title) => liveSink(run, id, title),
    login: deps.login,
    log: (level, message) => log(level, message, id),
    progress: (progress, message) =>
      emit({ type: "node:progress", runId, nodeId: id, progress, message }),
    step: (nodeId, status, message) => {
      if (status === "start") emit({ type: "node:start", runId, nodeId, at: Date.now() });
      else if (status === "error")
        emit({ type: "node:error", runId, nodeId, message: message ?? "Failed" });
      else emit(doneEvent(runId, nodeId, false));
    },
    compile: store.compile,
    results: store.results,
    secrets: deps.secrets,
  };
}

class RunState {
  readonly store: ResultStore;
  readonly failed = new Set<string>();
  readonly skipped = new Set<string>();
  readonly failures: Record<string, NodeFailure> = {};
  readonly outputs: string[] = [];
  private readonly tools: RunTools;
  private readonly cacheDir: string;

  constructor(
    private readonly run: RunContext,
    private readonly known: Map<string, Known>,
    private readonly forced: Set<string>,
  ) {
    const paths = projectPaths(run.req.projectDir);
    this.store = new ResultStore(run.req.workflow);
    this.cacheDir = paths.cache;
    this.tools = {
      ffmpeg: createFfmpegTools(run.deps.env.engineDir),
      engine: enginePaths(run.deps.env.engineDir),
      outDir: paths.out,
    };
  }

  dependencies(node: WorkflowNode, def: NodeDef, toRun: ReadonlySet<string>): string[] {
    const { workflow } = this.run.req;
    const id = node.id;
    const logins = isCapture(def) ? loginDependencies(workflow, id, this.run.vars) : [];
    return [...new Set([...upstreamOf(workflow, id), ...logins])].filter(
      (u) => u !== id && toRun.has(u),
    );
  }

  private blocked(id: string): boolean {
    return this.failed.has(id) || this.skipped.has(id);
  }

  private markChain(node: WorkflowNode) {
    if (!isCapture(this.known.get(node.id)?.def)) return;
    for (const up of upstreamOf(this.run.req.workflow, node.id))
      if (this.known.get(up)?.def.virtual) this.run.deps.emit(doneEvent(this.run.runId, up, true));
  }

  private collectOutputs(node: WorkflowNode, result: NodeResult) {
    if (!isOutputType(node.type)) return;
    for (const a of result.artifacts) this.outputs.push(a.path);
  }

  async runNode(job: Job): Promise<void> {
    const { node, def } = job;
    const { runId, deps } = this.run;
    if (deps.signal.aborted || this.skipIfBlocked(job)) return;
    if (!def.volatile && !this.forced.has(node.id) && (await this.fromCache(job))) return;
    try {
      const executor = getExecutor(node.type);
      if (!executor) throw new Error("This kind of node can't run.");
      const result = await this.executeJob(job, executor);
      this.store.set(node.id, result);
      if (!def.volatile) await writeCached(this.cacheDir, result);
      deps.emit(doneEvent(runId, node.id, false, result));
      this.collectOutputs(node, result);
    } catch (error) {
      this.fail(node.id, error);
    }
  }

  private skipIfBlocked({ node, needs }: Job): boolean {
    if (!needs.some((d) => this.blocked(d))) return false;
    this.skipped.add(node.id);
    this.run.deps.emit({
      type: "node:skipped",
      runId: this.run.runId,
      nodeId: node.id,
      reason: "Something before it failed",
    });
    return true;
  }

  private async executeJob({ node, hash }: Job, executor: Executor): Promise<NodeResult> {
    const { runId, req, deps, log } = this.run;
    const started = Date.now();
    deps.emit({ type: "node:start", runId, nodeId: node.id, at: started });
    const dir = path.join(this.cacheDir, hash);
    await mkdir(dir, { recursive: true });
    const ctx = executorContext(this.run, this.tools, this.store, { node, hash, dir });
    const out = await executor.run(ctx, this.store.inputsFor(node.id));
    const artifacts = await decorateArtifacts(
      req.projectDir,
      dir,
      this.tools.ffmpeg,
      out.artifacts ?? [],
      (level, m) => log(level, m, node.id),
    );
    return {
      hash,
      outputs: out.outputs,
      artifacts,
      finishedAt: Date.now(),
      durationMs: Date.now() - started,
      traceId: out.traceId,
    };
  }

  private async fromCache({ node, hash }: Job): Promise<boolean> {
    const cached = await readCached(this.cacheDir, hash, this.run.req.projectDir);
    if (!cached) return false;
    this.store.set(node.id, cached);
    this.markChain(node);
    this.run.deps.emit(doneEvent(this.run.runId, node.id, true, cached));
    this.collectOutputs(node, cached);
    return true;
  }

  private fail(id: string, error: unknown) {
    const { runId, deps } = this.run;
    this.failed.add(id);
    if (error instanceof ToolError) logWarning("run", `${id}: ${error.output}`);
    const traceId = error instanceof TracedError ? error.traceId : undefined;
    const message = deps.signal.aborted ? CANCELLED : errorMessage(error);
    deps.emit({
      type: "node:error",
      runId,
      nodeId: id,
      message,
      stack: error instanceof Error ? error.stack : undefined,
      traceId,
    });
    this.failures[id] = { message, at: Date.now(), traceId };
  }
}

type RunPlan = {
  known: Map<string, Known>;
  order: WorkflowNode[];
  needed: Set<string>;
  toRun: Known[];
};

async function planRun(run: RunContext): Promise<RunPlan> {
  const { workflow } = run.req;
  const known = knownNodes(workflow);
  const { sorted: order } = topoSort(workflow);
  const needed = await neededNodes(run, known, order);
  const runs = (node: WorkflowNode, entry: Known | undefined): entry is Known =>
    !!entry && needed.has(node.id) && !node.disabled && !isStructural(node) && !!entry.def.executes;
  const toRun = order.flatMap((node) => {
    const entry = known.get(node.id);
    return runs(node, entry) ? [entry] : [];
  });
  return { known, order, needed, toRun };
}

function reportInvalid({ runId, req, deps, log }: RunContext, plan: RunPlan) {
  const issues = workflowProblems(req.workflow).filter(
    (i) => i.level === "error" && (!i.nodeId || plan.needed.has(i.nodeId)),
  );
  if (!issues.length) return false;
  for (const i of issues) {
    if (i.nodeId) deps.emit({ type: "node:error", runId, nodeId: i.nodeId, message: i.message });
    else log("error", i.message);
  }
  deps.emit(
    runFailed(runId, issues[0].message, {
      failedNodeIds: [...new Set(issues.flatMap((i) => (i.nodeId ? [i.nodeId] : [])))],
    }),
  );
  return true;
}

function forcedNodes(req: RunRequest): Set<string> {
  const forced = new Set<string>(req.targetNodeIds ?? []);
  if (req.fromNodeId) {
    forced.add(req.fromNodeId);
    for (const d of downstreamOf(req.workflow, req.fromNodeId)) forced.add(d);
  }
  return forced;
}

async function finishRun(
  run: RunContext,
  plan: RunPlan,
  state: RunState,
  previous: LastRun,
  error: string | undefined,
) {
  const { runId, req, deps, log } = run;
  const ids = new Set(req.workflow.nodes.map((n) => n.id));
  await writeLastRun(
    req.projectDir,
    mergeLastRun(previous, state.store.results, state.failures, (id) => ids.has(id)),
  ).catch((e: unknown) => warnFailed(log, "Couldn't save the run results", e));

  const failedNodeIds = [...state.failed];
  const skipped = state.skipped.size;
  const nameOf = (id: string) => {
    const node = plan.known.get(id)?.node;
    return node ? nodeName(node) : id;
  };
  const anyFailed = failedNodeIds.length > 0 || skipped > 0;
  const { aborted } = deps.signal;
  deps.emit({
    type: "run:done",
    runId,
    ok: !aborted && !anyFailed && !error,
    cancelled: aborted,
    error: aborted
      ? undefined
      : (error ?? (anyFailed ? failureMessage(failedNodeIds.map(nameOf), skipped) : undefined)),
    failedNodeIds: aborted ? [] : failedNodeIds,
    skippedNodeIds: aborted ? [] : [...state.skipped],
    outputs: state.outputs,
    at: Date.now(),
  });
}

export async function runWorkflow(runId: string, req: RunRequest, deps: RunDeps): Promise<void> {
  const { workflow, projectDir } = req;
  const log: Log = (level, message, nodeId) =>
    deps.emit({ type: "node:log", runId, nodeId, level, message, at: Date.now() });
  const vars = workflowVars(workflow);
  const run: RunContext = { runId, req, deps, vars, log };

  const paths = projectPaths(projectDir);
  await mkdir(paths.cache, { recursive: true });
  await mkdir(paths.out, { recursive: true });

  const plan = await planRun(run);
  if (reportInvalid(run, plan)) return;
  const { known, toRun } = plan;

  deps.emit({ type: "run:start", runId, nodeIds: toRun.map((j) => j.node.id), at: Date.now() });

  const hashes = hashNodes(workflow, plan.order, vars, projectDir);
  const previous: LastRun = (await readLastRun(projectDir)) ?? { updatedAt: 0, nodes: {} };
  const state = new RunState(run, known, forcedNodes(req));
  for (const [id, r] of Object.entries(previous.nodes))
    if (hashes.get(id) === r.hash) state.store.set(id, r);

  const runSet = new Set(toRun.map((j) => j.node.id));
  const jobs = new Map<string, Job>();
  for (const entry of toRun)
    jobs.set(entry.node.id, {
      ...entry,
      hash: requireHash(hashes, entry.node.id),
      needs: state.dependencies(entry.node, entry.def, runSet),
    });
  const scheduler = new Scheduler(
    jobs,
    ({ node, def }) => def.mutates?.(parseNodeParams(def, node, vars)),
    (job) => state.runNode(job),
  );

  let error: string | undefined;
  try {
    await scheduler.all();
  } catch (e) {
    error = errorMessage(e);
  }
  await finishRun(run, plan, state, previous, error);
}
