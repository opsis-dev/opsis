import { mkdir } from "node:fs/promises";
import path from "node:path";
import pLimit, { type LimitFunction } from "p-limit";
import { compileWorkflow, workflowVars, type CompileResult } from "../../shared/compile/compile";
import {
  downstreamOf,
  isStructural,
  loginDependencies,
  nodesById,
  runTargets,
  topoSort,
  upstreamOf,
  validateWorkflow,
} from "../../shared/graph";
import { isOutputType } from "../../shared/nodes/output";
import type { RunRequest } from "../../shared/ipc";
import { parseParams, type NodeDef, type NodePhase } from "../../shared/nodes/define";
import { getNodeDef, nodeName } from "../../shared/nodes/registry";
import type { LogLevel, RunEvent } from "../../shared/schema/events";
import type { CompileValue, LastRun, NodeFailure, NodeResult } from "../../shared/schema/values";
import type { Workflow, WorkflowNode } from "../../shared/schema/workflow";
import { enginePaths } from "../../engine/paths";
import type { LiveMessage } from "../../shared/live";
import { secretName } from "../../shared/secrets";
import { interpolate } from "../../shared/vars";
import { failureMessage, runFailed } from "../../shared/run-progress";
import type { LoginRequest, WorkerEnv } from "../../shared/worker-protocol";
import type { LiveSink } from "../capture/take-run";
import { getExecutor } from "../executors";
import { createFfmpegTools, ToolError } from "../media/ffmpeg";
import { nodeHash, readCached, writeCached, writeLastRun } from "./cache";
import { projectPaths, readLastRun, toPosixRel } from "../../engine/project-paths";
import { TracedError, type ExecutorContext } from "./context";
import { decorateArtifacts } from "./posters";
import { CANCELLED, errorMessage } from "../../shared/errors";

export type RunDeps = {
  env: WorkerEnv;
  emit: (event: RunEvent) => void;
  signal: AbortSignal;
  secrets: ExecutorContext["secrets"];
  login: (request: LoginRequest) => Promise<string>;
  live: ((message: LiveMessage) => void) | null;
};

type Log = (level: LogLevel, message: string, nodeId?: string) => void;

type Known = { node: WorkflowNode; def: NodeDef };

type Job = Known & { hash: string; needs: string[] };

type Results = Partial<Record<string, NodeResult>>;

const isCapture = (def: NodeDef | undefined) => def?.category === "Capture";

function knownNodes(workflow: Workflow): Map<string, Known> {
  const known = new Map<string, Known>();
  for (const node of workflow.nodes) {
    const def = getNodeDef(node.type);
    if (def) known.set(node.id, { node, def });
  }
  return known;
}

async function signInSaved(
  node: WorkflowNode,
  vars: Record<string, string>,
  deps: RunDeps,
  log: Log,
) {
  const raw = interpolate(node.params, vars).secretName;
  const name = secretName(typeof raw === "string" ? raw : undefined);
  const saved = await deps.secrets.read(name).catch((error: unknown) => {
    log(
      "warn",
      `Couldn't read the saved sign-in “${name}”, so it will be asked for again: ${errorMessage(error)}`,
    );
    return null;
  });
  return !!saved;
}

async function neededNodes(
  workflow: Workflow,
  req: RunRequest,
  known: Map<string, Known>,
  order: string[],
  vars: Record<string, string>,
  deps: RunDeps,
  log: Log,
): Promise<Set<string>> {
  const targets = req.targetNodeIds?.length ? req.targetNodeIds : runTargets(workflow);
  const needed = new Set<string>();
  for (const id of targets) {
    needed.add(id);
    for (const up of upstreamOf(workflow, id)) needed.add(up);
  }
  const captures = order.filter((id) => needed.has(id) && isCapture(known.get(id)?.def));
  for (const id of captures) {
    for (const login of loginDependencies(workflow, id, vars)) {
      const node = known.get(login)?.node;
      if (needed.has(login) || !node) continue;
      if (!(await signInSaved(node, vars, deps, log))) needed.add(login);
    }
  }
  return needed;
}

function hashNodes(
  workflow: Workflow,
  order: string[],
  vars: Record<string, string>,
): Map<string, string> {
  const nodes = nodesById(workflow.nodes);
  const hashes = new Map<string, string>();
  for (const id of order) {
    const node = nodes.get(id);
    if (!node) continue;
    const upstream = workflow.edges
      .filter((e) => e.target === id && hashes.has(e.source))
      .map((e): [string, string] => [e.targetHandle, `${hashes.get(e.source)}:${e.sourceHandle}`]);
    const params = interpolate(node.params, vars);
    const keyed =
      getNodeDef(node.type)?.phase === "render"
        ? { params, settings: workflow.settings, mix: workflow.mix ?? null }
        : params;
    hashes.set(id, nodeHash(node.type, keyed, upstream));
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
  results: Results,
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
  private readonly lanes: Record<NodePhase, LimitFunction> = {
    capture: pLimit(3),
    media: pLimit(4),
    compile: pLimit(8),
    render: pLimit(1),
    post: pLimit(2),
  };
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

class RunState {
  readonly results: Results = {};
  private resultsVersion = 0;
  private compiled: { version: number; result: CompileResult } | null = null;
  readonly failed = new Set<string>();
  readonly skipped = new Set<string>();
  readonly failures: Record<string, NodeFailure> = {};
  readonly outputs: string[] = [];
  private readonly ffmpeg;
  private readonly engine;
  private readonly paths;

  constructor(
    private readonly runId: string,
    private readonly req: RunRequest,
    private readonly deps: RunDeps,
    private readonly vars: Record<string, string>,
    private readonly known: Map<string, Known>,
    private readonly forced: Set<string>,
    private readonly log: Log,
  ) {
    this.ffmpeg = createFfmpegTools(deps.env.engineDir);
    this.engine = enginePaths(deps.env.engineDir);
    this.paths = projectPaths(req.projectDir);
  }

  private emit(event: RunEvent) {
    this.deps.emit(event);
  }

  dependencies(node: WorkflowNode, def: NodeDef, toRun: ReadonlySet<string>): string[] {
    const { workflow } = this.req;
    const id = node.id;
    const logins = isCapture(def) ? loginDependencies(workflow, id, this.vars) : [];
    return [...new Set([...upstreamOf(workflow, id), ...logins])].filter(
      (u) => u !== id && toRun.has(u),
    );
  }

  setResult(id: string, result: NodeResult) {
    this.results[id] = result;
    this.resultsVersion++;
  }

  private doneEvent(nodeId: string, cached: boolean): RunEvent {
    return { type: "node:done", runId: this.runId, nodeId, cached };
  }

  private markChain(node: WorkflowNode) {
    if (!isCapture(getNodeDef(node.type))) return;
    for (const up of upstreamOf(this.req.workflow, node.id))
      if (this.known.get(up)?.def.virtual) this.emit(this.doneEvent(up, true));
  }

  private collectOutputs(node: WorkflowNode, result: NodeResult) {
    if (!isOutputType(node.type)) return;
    for (const a of result.artifacts) this.outputs.push(a.path);
  }

  async runNode(job: Job): Promise<void> {
    const { node, def, hash } = job;
    const id = node.id;
    if (this.deps.signal.aborted) return;
    if (job.needs.some((d) => this.failed.has(d))) {
      this.failed.add(id);
      this.skipped.add(id);
      this.emit({
        type: "node:skipped",
        runId: this.runId,
        nodeId: id,
        reason: "Something before it failed",
      });
      return;
    }
    if (!def.volatile && !this.forced.has(id) && (await this.fromCache(job))) return;

    const executor = getExecutor(node.type);
    if (!executor) {
      this.failed.add(id);
      this.emit({
        type: "node:error",
        runId: this.runId,
        nodeId: id,
        message: `No executor for ${node.type}`,
      });
      return;
    }

    const started = Date.now();
    this.emit({ type: "node:start", runId: this.runId, nodeId: id, at: started });
    const dir = path.join(this.paths.cache, hash);
    await mkdir(dir, { recursive: true });
    const ctx = this.contextFor(node, hash, dir);
    const inputs = this.inputsFor(id);
    try {
      const out = await executor.run(ctx, inputs, parseParams(def, node, this.vars));
      const result: NodeResult = {
        hash,
        outputs: out.outputs,
        artifacts: await decorateArtifacts(
          this.req.projectDir,
          dir,
          this.ffmpeg,
          out.artifacts ?? [],
          (m) => this.log("warn", m, id),
        ),
        finishedAt: Date.now(),
        durationMs: Date.now() - started,
        ...(out.traceId ? { traceId: out.traceId } : {}),
      };
      this.setResult(id, result);
      if (!def.volatile) await writeCached(this.paths.cache, result);
      this.emit({ type: "node:done", runId: this.runId, nodeId: id, cached: false, result });
      this.collectOutputs(node, result);
    } catch (error) {
      this.fail(id, error);
    }
  }

  private async fromCache({ node, hash }: Job): Promise<boolean> {
    const cached = await readCached(this.paths.cache, hash, this.req.projectDir);
    if (!cached) return false;
    this.setResult(node.id, cached);
    this.markChain(node);
    this.emit({
      type: "node:done",
      runId: this.runId,
      nodeId: node.id,
      cached: true,
      result: cached,
    });
    this.collectOutputs(node, cached);
    return true;
  }

  private fail(id: string, error: unknown) {
    this.failed.add(id);
    if (error instanceof ToolError) this.log("error", error.output, id);
    const traceId = error instanceof TracedError ? error.traceId : undefined;
    const message = this.deps.signal.aborted ? CANCELLED : errorMessage(error);
    this.emit({
      type: "node:error",
      runId: this.runId,
      nodeId: id,
      message,
      stack: error instanceof Error ? error.stack : undefined,
      traceId,
    });
    this.failures[id] = { message, at: Date.now(), ...(traceId ? { traceId } : {}) };
  }

  private compile = (): CompileResult => {
    if (this.compiled?.version !== this.resultsVersion)
      this.compiled = {
        version: this.resultsVersion,
        result: compileWorkflow(this.req.workflow, { results: this.results }),
      };
    return this.compiled.result;
  };

  private inputsFor(id: string): Partial<Record<string, CompileValue>> {
    const inputs: Partial<Record<string, CompileValue>> = {};
    const values: Partial<CompileResult["values"]> = this.compile().values;
    for (const e of this.req.workflow.edges) {
      if (e.target !== id) continue;
      const v =
        this.results[e.source]?.outputs[e.sourceHandle] ?? values[e.source]?.[e.sourceHandle];
      if (v) inputs[e.targetHandle] = v;
    }
    return inputs;
  }

  private contextFor(node: WorkflowNode, hash: string, dir: string): ExecutorContext {
    const { runId, req, deps } = this;
    const id = node.id;
    const relDir = toPosixRel(req.projectDir, dir);
    return {
      runId,
      node,
      workflow: req.workflow,
      env: deps.env,
      projectDir: req.projectDir,
      dir,
      relDir,
      outDir: this.paths.out,
      hash,
      vars: this.vars,
      signal: deps.signal,
      ffmpeg: this.ffmpeg,
      engine: this.engine,
      live: (title) => this.liveSink(id, title),
      login: deps.login,
      log: (level, message) => this.log(level, message, id),
      progress: (progress, message) =>
        this.emit({ type: "node:progress", runId, nodeId: id, progress, message }),
      step: (nodeId, status, message) => {
        if (status === "start") this.emit({ type: "node:start", runId, nodeId, at: Date.now() });
        else if (status === "error")
          this.emit({ type: "node:error", runId, nodeId, message: message ?? "Failed" });
        else this.emit(this.doneEvent(nodeId, false));
      },
      compile: this.compile,
      results: this.results,
      secrets: deps.secrets,
    };
  }

  private liveSink(nodeId: string, title: string): LiveSink | null {
    const send = this.deps.live;
    if (!send) return null;
    const captureId = `${this.runId}:${nodeId}`;
    const minGap = 1000 / Math.max(1, this.deps.env.settings.live.fps);
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
}

type RunPlan = {
  known: Map<string, Known>;
  vars: Record<string, string>;
  order: string[];
  needed: Set<string>;
  toRun: string[];
};

async function planRun(req: RunRequest, deps: RunDeps, log: Log): Promise<RunPlan> {
  const { workflow } = req;
  const known = knownNodes(workflow);
  const vars = workflowVars(workflow);
  const { order } = topoSort(workflow);
  const needed = await neededNodes(workflow, req, known, order, vars, deps, log);
  const executes = (n: WorkflowNode, def: NodeDef) =>
    !n.disabled && !isStructural(n) && !!def.executes;
  const toRun = order.filter((id) => {
    const entry = known.get(id);
    return needed.has(id) && !!entry && executes(entry.node, entry.def);
  });
  return { known, vars, order, needed, toRun };
}

function reportInvalid(runId: string, req: RunRequest, plan: RunPlan, deps: RunDeps, log: Log) {
  const issues = validateWorkflow(req.workflow).filter(
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
  runId: string,
  req: RunRequest,
  deps: RunDeps,
  state: RunState,
  previous: LastRun,
  plan: RunPlan,
  error: string | undefined,
  log: Log,
) {
  const { workflow, projectDir } = req;
  const ids = new Set(workflow.nodes.map((n) => n.id));
  await writeLastRun(
    projectDir,
    mergeLastRun(previous, state.results, state.failures, (id) => ids.has(id)),
  ).catch((e: unknown) => log("warn", `Couldn't save the run results: ${errorMessage(e)}`));

  const { failed, skipped } = state;
  const failedNodeIds = [...failed].filter((id) => !skipped.has(id));
  const nameOf = (id: string) => {
    const node = plan.known.get(id)?.node;
    return node ? nodeName(node) : id;
  };
  const { aborted } = deps.signal;
  deps.emit({
    type: "run:done",
    runId,
    ok: !aborted && failed.size === 0 && !error,
    cancelled: aborted,
    error:
      error ?? (failed.size ? failureMessage(failedNodeIds.map(nameOf), skipped.size) : undefined),
    failedNodeIds,
    skippedNodeIds: [...skipped],
    outputs: state.outputs,
    at: Date.now(),
  });
}

export async function runWorkflow(runId: string, req: RunRequest, deps: RunDeps): Promise<void> {
  const { workflow, projectDir } = req;
  const log: Log = (level, message, nodeId) =>
    deps.emit({ type: "node:log", runId, nodeId, level, message, at: Date.now() });

  const paths = projectPaths(projectDir);
  await mkdir(paths.cache, { recursive: true });
  await mkdir(paths.out, { recursive: true });

  const plan = await planRun(req, deps, log);
  if (reportInvalid(runId, req, plan, deps, log)) return;
  const { known, vars, toRun } = plan;

  deps.emit({ type: "run:start", runId, nodeIds: toRun, at: Date.now() });

  const hashes = hashNodes(workflow, plan.order, vars);
  const previous: LastRun = (await readLastRun(projectDir)) ?? { updatedAt: 0, nodes: {} };
  const state = new RunState(runId, req, deps, vars, known, forcedNodes(req), log);
  for (const [id, r] of Object.entries(previous.nodes))
    if (hashes.get(id) === r.hash) state.setResult(id, r);

  const runSet = new Set(toRun);
  const jobs = new Map<string, Job>();
  for (const id of toRun) {
    const entry = known.get(id);
    if (entry)
      jobs.set(id, {
        ...entry,
        hash: requireHash(hashes, id),
        needs: state.dependencies(entry.node, entry.def, runSet),
      });
  }
  const scheduler = new Scheduler(
    jobs,
    ({ node, def }) => def.mutates?.(parseParams(def, node, vars)),
    (job) => state.runNode(job),
  );

  let error: string | undefined;
  try {
    await scheduler.all();
  } catch (e) {
    error = errorMessage(e);
  }
  await finishRun(runId, req, deps, state, previous, plan, error, log);
}
