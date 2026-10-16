import { mkdir } from "node:fs/promises";
import path from "node:path";
import pLimit from "p-limit";
import { compileWorkflow, workflowVars } from "../../shared/compile/compile";
import {
  downstreamOf,
  isStructural,
  loginDependencies,
  runTargets,
  topoSort,
  upstreamOf,
  validateWorkflow,
} from "../../shared/graph";
import type { RunRequest } from "../../shared/ipc";
import type { NodePhase } from "../../shared/nodes/define";
import { getNodeDef } from "../../shared/nodes/registry";
import type { LogLevel, RunEvent } from "../../shared/schema/events";
import type { CompileValue, LastRun, NodeFailure, NodeResult } from "../../shared/schema/values";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { enginePaths } from "../../engine/paths";
import type { LiveMessage } from "../../shared/live";
import { interpolate } from "../../shared/vars";
import { failureMessage } from "../../shared/run-progress";
import type { LoginRequest, WorkerEnv } from "../../shared/worker-protocol";
import type { LiveSink } from "../capture/driver";
import { getExecutor } from "../executors";
import { createFfmpegTools } from "../media/ffmpeg";
import {
  nodeHash,
  projectPaths,
  readCached,
  readLastRun,
  writeCached,
  writeLastRun,
} from "./cache";
import { TracedError, type ExecutorContext } from "./context";
import { decorateArtifacts } from "./posters";

const LANE_LIMITS: Record<NodePhase, number> = {
  capture: 3,
  media: 4,
  compile: 8,
  render: 1,
  post: 2,
};

export type RunDeps = {
  env: WorkerEnv;
  emit: (event: RunEvent) => void;
  signal: AbortSignal;
  secrets: ExecutorContext["secrets"];
  login: (request: LoginRequest) => Promise<string>;
  live: ((message: LiveMessage) => void) | null;
};

export async function runWorkflow(runId: string, req: RunRequest, deps: RunDeps): Promise<void> {
  const { workflow, projectDir } = req;
  const { emit, signal } = deps;
  const log = (level: LogLevel, message: string, nodeId?: string) =>
    emit({ type: "node:log", runId, nodeId, level, message, at: Date.now() });

  const paths = projectPaths(projectDir);
  await mkdir(paths.cache, { recursive: true });
  await mkdir(paths.out, { recursive: true });

  const byId = new Map(workflow.nodes.map((n) => [n.id, n]));
  const vars = workflowVars(workflow);
  const { order } = topoSort(workflow);

  const executes = (n: WorkflowNode | undefined) =>
    !!n && !n.disabled && !isStructural(n) && !!getNodeDef(n.type)?.executes;
  const finalTargets = req.targetNodeIds?.length ? req.targetNodeIds : runTargets(workflow);
  const needed = new Set<string>();
  for (const id of finalTargets) {
    needed.add(id);
    for (const up of upstreamOf(workflow, id)) needed.add(up);
  }
  const captures = order.filter(
    (id) => needed.has(id) && getNodeDef(byId.get(id)?.type ?? "")?.category === "Capture",
  );
  for (const id of captures) {
    for (const login of loginDependencies(workflow, id, vars)) {
      if (needed.has(login)) continue;
      const params = interpolate(byId.get(login)!.params ?? {}, vars) as { secretName?: string };
      const saved = await deps.secrets
        .read(params.secretName?.trim() || "default")
        .catch(() => null);
      if (!saved) needed.add(login);
    }
  }
  const toRun = order.filter((id) => needed.has(id) && executes(byId.get(id)));

  const issues = validateWorkflow(workflow).filter(
    (i) => i.level === "error" && (!i.nodeId || needed.has(i.nodeId)),
  );
  if (issues.length) {
    for (const i of issues) {
      if (i.nodeId) emit({ type: "node:error", runId, nodeId: i.nodeId, message: i.message });
      else log("error", i.message);
    }
    emit({
      type: "run:done",
      runId,
      ok: false,
      cancelled: false,
      error: issues[0].message,
      failedNodeIds: [...new Set(issues.flatMap((i) => (i.nodeId ? [i.nodeId] : [])))],
      outputs: [],
      at: Date.now(),
    });
    return;
  }

  emit({ type: "run:start", runId, nodeIds: toRun, at: Date.now() });

  const hashes = new Map<string, string>();
  for (const id of order) {
    const node = byId.get(id)!;
    const upstream = workflow.edges
      .filter((e) => e.target === id && hashes.has(e.source))
      .map(
        (e) => [e.targetHandle, `${hashes.get(e.source)}:${e.sourceHandle}`] as [string, string],
      );
    const params = interpolate(node.params ?? {}, vars);
    const keyed =
      getNodeDef(node.type)?.phase === "render"
        ? { params, settings: workflow.settings, mix: workflow.mix ?? null }
        : params;
    hashes.set(id, nodeHash(node.type, keyed, upstream));
  }

  const forced = new Set<string>(req.targetNodeIds ?? []);
  if (req.fromNodeId) {
    forced.add(req.fromNodeId);
    for (const d of downstreamOf(workflow, req.fromNodeId)) forced.add(d);
  }

  const previous: LastRun = (await readLastRun(projectDir)) ?? { updatedAt: 0, nodes: {} };
  const results: Record<string, NodeResult> = {};
  for (const [id, r] of Object.entries(previous.nodes))
    if (hashes.get(id) === r.hash) results[id] = r;

  const ffmpeg = createFfmpegTools(deps.env.engineDir);
  const engine = enginePaths(deps.env.engineDir);
  const liveSink = (nodeId: string, title: string): LiveSink | null => {
    const send = deps.live;
    if (!send) return null;
    const captureId = `${runId}:${nodeId}`;
    const minGap = 1000 / Math.max(1, deps.env.settings.live.fps);
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
  };
  const lanes = Object.fromEntries(
    Object.entries(LANE_LIMITS).map(([k, v]) => [k, pLimit(v)]),
  ) as Record<NodePhase, ReturnType<typeof pLimit>>;
  const serial = new Map<string, Promise<unknown>>();
  const failed = new Set<string>();
  const skipped = new Set<string>();
  const failures: Record<string, NodeFailure> = {};
  const outputs: string[] = [];
  const promises = new Map<string, Promise<void>>();

  const execDeps = (id: string) =>
    [
      ...new Set([
        ...upstreamOf(workflow, id),
        ...(getNodeDef(byId.get(id)?.type ?? "")?.category === "Capture"
          ? loginDependencies(workflow, id, vars)
          : []),
      ]),
    ].filter((u) => u !== id && toRun.includes(u));

  const runNode = async (id: string): Promise<void> => {
    const node = byId.get(id)!;
    const def = getNodeDef(node.type)!;
    const hash = hashes.get(id)!;
    if (signal.aborted) return;
    if (execDeps(id).some((d) => failed.has(d))) {
      failed.add(id);
      skipped.add(id);
      emit({ type: "node:skipped", runId, nodeId: id, reason: "Something before it failed" });
      return;
    }

    if (!def.volatile && !forced.has(id)) {
      const cached = await readCached(paths.cache, hash, projectDir);
      if (cached) {
        results[id] = cached;
        markChain(node, "done");
        emit({ type: "node:done", runId, nodeId: id, cached: true, result: cached });
        collectOutputs(node, cached);
        return;
      }
    }

    const executor = getExecutor(node.type);
    if (!executor) {
      failed.add(id);
      emit({ type: "node:error", runId, nodeId: id, message: `No executor for ${node.type}` });
      return;
    }

    const started = Date.now();
    emit({ type: "node:start", runId, nodeId: id, at: started });
    const dir = path.join(paths.cache, hash);
    await mkdir(dir, { recursive: true });
    const relDir = path.relative(projectDir, dir).split(path.sep).join("/");

    const inputs: Partial<Record<string, CompileValue>> = {};
    const compiled = () => compileWorkflow(workflow, { results });
    const values = compiled().values;
    for (const e of workflow.edges) {
      if (e.target !== id) continue;
      const v = results[e.source]?.outputs[e.sourceHandle] ?? values[e.source]?.[e.sourceHandle];
      if (v) inputs[e.targetHandle] = v;
    }

    const ctx: ExecutorContext = {
      runId,
      node,
      workflow,
      env: deps.env,
      projectDir,
      dir,
      relDir,
      outDir: paths.out,
      hash,
      vars,
      signal,
      ffmpeg,
      engine,
      settings: deps.env.settings,
      live: (title) => liveSink(id, title),
      login: deps.login,
      log: (level, message) => log(level, message, id),
      progress: (progress, message) =>
        emit({ type: "node:progress", runId, nodeId: id, progress, message }),
      step: (nodeId, status, message) => {
        if (status === "start") emit({ type: "node:start", runId, nodeId, at: Date.now() });
        else if (status === "error")
          emit({ type: "node:error", runId, nodeId, message: message ?? "Failed" });
        else
          emit({
            type: "node:done",
            runId,
            nodeId,
            cached: false,
            result: {
              hash: hashes.get(nodeId) ?? "",
              outputs: {},
              artifacts: [],
              finishedAt: Date.now(),
              durationMs: 0,
            },
          });
      },
      compile: compiled,
      results,
      secrets: deps.secrets,
      ref: (file) => `project:${relDir}/${file}`,
    };

    try {
      const params = def.params.parse(interpolate(node.params ?? {}, vars));
      const out = await executor.run(ctx, inputs, params);
      const result: NodeResult = {
        hash,
        outputs: out.outputs,
        artifacts: await decorateArtifacts(projectDir, dir, ffmpeg, out.artifacts ?? []),
        finishedAt: Date.now(),
        durationMs: Date.now() - started,
        ...(out.traceId ? { traceId: out.traceId } : {}),
      };
      results[id] = result;
      if (!def.volatile) await writeCached(paths.cache, result);
      emit({ type: "node:done", runId, nodeId: id, cached: false, result });
      collectOutputs(node, result);
    } catch (error) {
      failed.add(id);
      const traceId = error instanceof TracedError ? error.traceId : undefined;
      const message = signal.aborted
        ? "Cancelled"
        : error instanceof Error
          ? error.message
          : String(error);
      emit({
        type: "node:error",
        runId,
        nodeId: id,
        message,
        stack: error instanceof Error ? error.stack : undefined,
        traceId,
      });
      failures[id] = { message, at: Date.now(), ...(traceId ? { traceId } : {}) };
    }
  };

  const markChain = (node: WorkflowNode, status: "done") => {
    if (getNodeDef(node.type)?.category !== "Capture") return;
    for (const up of upstreamOf(workflow, node.id)) {
      const n = byId.get(up);
      if (n && getNodeDef(n.type)?.virtual) {
        emit({
          type: "node:done",
          runId,
          nodeId: up,
          cached: true,
          result: {
            hash: hashes.get(up) ?? "",
            outputs: {},
            artifacts: [],
            finishedAt: Date.now(),
            durationMs: 0,
          },
        });
      }
    }
    void status;
  };

  const collectOutputs = (node: WorkflowNode, result: NodeResult) => {
    if (!node.type.startsWith("output.")) return;
    for (const a of result.artifacts) outputs.push(a.path);
  };

  const schedule = (id: string): Promise<void> => {
    const existing = promises.get(id);
    if (existing) return existing;
    const node = byId.get(id)!;
    const def = getNodeDef(node.type)!;
    const p = (async () => {
      await Promise.all(execDeps(id).map(schedule));
      const lane = lanes[def.phase];
      const key = def.mutates?.(def.params.parse(interpolate(node.params ?? {}, vars)) as never);
      if (key) {
        const prev = serial.get(key) ?? Promise.resolve();
        const next = prev.then(() => lane(() => runNode(id)));
        serial.set(
          key,
          next.catch(() => undefined),
        );
        await next;
      } else {
        await lane(() => runNode(id));
      }
    })();
    promises.set(id, p);
    return p;
  };

  let error: string | undefined;
  try {
    await Promise.all(toRun.map(schedule));
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  }

  const merged: LastRun = {
    updatedAt: Date.now(),
    nodes: { ...previous.nodes },
    failures: { ...previous.failures, ...failures },
  };
  for (const [id, r] of Object.entries(results)) {
    merged.nodes[id] = r;
    delete merged.failures![id];
  }
  for (const id of Object.keys(merged.nodes)) if (!byId.has(id)) delete merged.nodes[id];
  for (const id of Object.keys(merged.failures!)) if (!byId.has(id)) delete merged.failures![id];
  await writeLastRun(projectDir, merged).catch((e: unknown) =>
    log("warn", `Couldn't save the run results: ${String(e)}`),
  );

  const ok = !signal.aborted && failed.size === 0 && !error;
  const failedNodeIds = [...failed].filter((id) => !skipped.has(id));
  const nameOf = (id: string) => {
    const node = byId.get(id);
    return node?.label || getNodeDef(node?.type ?? "")?.title || id;
  };
  emit({
    type: "run:done",
    runId,
    ok,
    cancelled: signal.aborted,
    error:
      error ?? (failed.size ? failureMessage(failedNodeIds.map(nameOf), skipped.size) : undefined),
    failedNodeIds,
    skippedNodeIds: [...skipped],
    outputs,
    at: Date.now(),
  });
}
