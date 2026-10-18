import { isOutputType } from "./nodes/output";
import type { NodeRunStatus, RunEvent } from "./schema/events";
import type { Artifact, NodeResult } from "./schema/values";

type NodeProgress = { value: number; message?: string };

export type RunProgress = {
  runId: string | null;
  statuses: Partial<Record<string, NodeRunStatus>>;
  progress: Partial<Record<string, NodeProgress>>;
  cached: Partial<Record<string, boolean>>;
  errors: Partial<Record<string, string>>;
  traces: Partial<Record<string, string>>;
  results: Partial<Record<string, NodeResult>>;
};

export const emptyRunProgress = (runId: string | null = null): RunProgress => ({
  runId,
  statuses: {},
  progress: {},
  cached: {},
  errors: {},
  traces: {},
  results: {},
});

const without = <T>(record: Record<string, T>, ids: readonly string[]): Record<string, T> => {
  const drop = new Set(ids);
  return Object.fromEntries(Object.entries(record).filter(([id]) => !drop.has(id)));
};

type RerunFields = Pick<RunProgress, "progress" | "cached" | "errors">;

const forgetNodes = (state: RerunFields, ids: readonly string[]): RerunFields => ({
  progress: without(state.progress, ids),
  cached: without(state.cached, ids),
  errors: without(state.errors, ids),
});

export function applyRunEvent<S extends RunProgress>(state: S, e: RunEvent): S {
  switch (e.type) {
    case "run:start":
      return {
        ...state,
        runId: e.runId,
        statuses: {
          ...state.statuses,
          ...Object.fromEntries(e.nodeIds.map((id) => [id, "queued" as const])),
        },
        ...forgetNodes(state, e.nodeIds),
      };
    case "node:start":
      return { ...state, statuses: { ...state.statuses, [e.nodeId]: "running" } };
    case "node:progress":
      return {
        ...state,
        progress: {
          ...state.progress,
          [e.nodeId]: {
            value: e.progress,
            ...(e.message === undefined ? {} : { message: e.message }),
          },
        },
      };
    case "node:done": {
      const { result } = e;
      const keep =
        !!result && (Object.keys(result.outputs).length > 0 || result.artifacts.length > 0);
      return {
        ...state,
        statuses: { ...state.statuses, [e.nodeId]: e.cached ? "cached" : "done" },
        cached: { ...state.cached, [e.nodeId]: e.cached },
        progress: { ...state.progress, [e.nodeId]: { value: 1 } },
        results: keep ? { ...state.results, [e.nodeId]: result } : state.results,
        traces: result?.traceId ? { ...state.traces, [e.nodeId]: result.traceId } : state.traces,
      };
    }
    case "node:error":
      return {
        ...state,
        statuses: { ...state.statuses, [e.nodeId]: "error" },
        errors: { ...state.errors, [e.nodeId]: e.message },
        traces: e.traceId ? { ...state.traces, [e.nodeId]: e.traceId } : state.traces,
      };
    case "node:skipped":
      return { ...state, statuses: { ...state.statuses, [e.nodeId]: "skipped" } };
    case "node:log":
    case "run:done":
      return state;
  }
}

export function resumeRunProgress<S extends RunProgress>(state: S, saved: RunProgress): S {
  const kept = forgetNodes(state, Object.keys(saved.statuses));
  return {
    ...state,
    runId: saved.runId,
    statuses: { ...state.statuses, ...saved.statuses },
    progress: { ...kept.progress, ...saved.progress },
    cached: { ...kept.cached, ...saved.cached },
    errors: { ...kept.errors, ...saved.errors },
    traces: { ...state.traces, ...saved.traces },
    results: { ...state.results, ...saved.results },
  };
}

export function failureMessage(failedNames: readonly string[], skipped: number): string {
  const skippedText = skipped ? `, ${skipped} skipped` : "";
  if (failedNames.length === 0)
    return skipped === 1 ? "1 node was skipped" : `${skipped} nodes were skipped`;
  if (failedNames.length === 1) return `${failedNames[0]} failed${skippedText}`;
  return `${failedNames.length} nodes failed: ${failedNames.join(", ")}${skippedText}`;
}

export function exportedArtifacts(
  nodes: ReadonlyArray<{ id: string; type: string }>,
  results: Partial<Record<string, NodeResult>>,
): Artifact[] {
  return nodes
    .filter((n) => isOutputType(n.type))
    .flatMap((n) => results[n.id] ?? [])
    .filter((r) => r.artifacts.length)
    .toSorted((a, b) => b.finishedAt - a.finishedAt)
    .flatMap((r) => r.artifacts)
    .filter((a, i, all) => all.findIndex((b) => b.path === a.path) === i);
}
