import { create } from "zustand";
import type { LogLevel, NodeRunStatus, RunEvent } from "@/shared/schema/events";
import type { LastRun, NodeResult } from "@/shared/schema/values";

export type LogLine = { at: number; level: LogLevel; nodeId?: string; message: string };

type RunState = {
  runId: string | null;
  running: boolean;
  statuses: Record<string, NodeRunStatus>;
  progress: Record<string, { value: number; message?: string }>;
  cached: Record<string, boolean>;
  errors: Record<string, string>;
  traces: Record<string, string>;
  logs: LogLine[];
  results: Record<string, NodeResult>;
  outputs: string[];
  lastError: string | null;
  seed: (lastRun: LastRun | null) => void;
  start: (runId: string) => void;
  apply: (event: RunEvent) => void;
  clearLogs: () => void;
};

export const useRunStore = create<RunState>()((set) => ({
  runId: null,
  running: false,
  statuses: {},
  progress: {},
  cached: {},
  errors: {},
  traces: {},
  logs: [],
  results: {},
  outputs: [],
  lastError: null,

  seed: (lastRun) => {
    const newerFailures = Object.entries(lastRun?.failures ?? {}).filter(
      ([id, f]) => f.at > (lastRun?.nodes[id]?.finishedAt ?? 0),
    );
    set({
      traces: Object.fromEntries([
        ...Object.entries(lastRun?.nodes ?? {}).flatMap(([id, r]) =>
          r.traceId ? [[id, r.traceId]] : [],
        ),
        ...newerFailures.flatMap(([id, f]) => (f.traceId ? [[id, f.traceId]] : [])),
      ]),
      results: lastRun?.nodes ?? {},
      statuses: {},
      progress: {},
      cached: {},
      errors: Object.fromEntries(newerFailures.map(([id, f]) => [id, f.message])),
      logs: [],
      outputs: [],
      lastError: null,
      running: false,
      runId: null,
    });
  },

  start: (runId) =>
    set({
      runId,
      running: true,
      statuses: {},
      progress: {},
      cached: {},
      errors: {},
      outputs: [],
      lastError: null,
    }),

  apply: (e) =>
    set((s) => {
      switch (e.type) {
        case "run:start":
          return {
            runId: e.runId,
            running: true,
            statuses: Object.fromEntries(e.nodeIds.map((id) => [id, "queued" as const])),
          };
        case "node:start":
          return { statuses: { ...s.statuses, [e.nodeId]: "running" } };
        case "node:progress":
          return {
            progress: { ...s.progress, [e.nodeId]: { value: e.progress, message: e.message } },
          };
        case "node:log":
          return {
            logs: [
              ...s.logs.slice(-1999),
              { at: e.at, level: e.level, nodeId: e.nodeId, message: e.message },
            ],
          };
        case "node:done": {
          const keep = Object.keys(e.result.outputs).length > 0 || e.result.artifacts.length > 0;
          return {
            statuses: { ...s.statuses, [e.nodeId]: e.cached ? "cached" : "done" },
            cached: { ...s.cached, [e.nodeId]: e.cached },
            progress: { ...s.progress, [e.nodeId]: { value: 1 } },
            results: keep ? { ...s.results, [e.nodeId]: e.result } : s.results,
            traces: e.result.traceId ? { ...s.traces, [e.nodeId]: e.result.traceId } : s.traces,
          };
        }
        case "node:error":
          return {
            statuses: { ...s.statuses, [e.nodeId]: "error" },
            errors: { ...s.errors, [e.nodeId]: e.message },
            traces: e.traceId ? { ...s.traces, [e.nodeId]: e.traceId } : s.traces,
            logs: [
              ...s.logs,
              { at: Date.now(), level: "error", nodeId: e.nodeId, message: e.message },
            ],
          };
        case "node:skipped":
          return { statuses: { ...s.statuses, [e.nodeId]: "skipped" } };
        case "run:done":
          return {
            running: false,
            outputs: e.outputs,
            lastError: e.ok ? null : (e.error ?? (e.cancelled ? "Cancelled" : "Failed")),
          };
        default:
          return {};
      }
    }),

  clearLogs: () => set({ logs: [] }),
}));
