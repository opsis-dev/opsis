import { create } from "zustand";
import {
  applyRunEvent,
  emptyRunProgress,
  resumeRunProgress,
  type RunProgress,
} from "@/shared/run-progress";
import type { LogLevel, RunEvent } from "@/shared/schema/events";
import type { LastRun } from "@/shared/schema/values";

export type LogLine = {
  id: number;
  at: number;
  level: LogLevel;
  nodeId?: string;
  message: string;
};

let logCount = 0;
const logLine = (line: Omit<LogLine, "id">): LogLine => ({ id: ++logCount, ...line });

type RunState = RunProgress & {
  running: boolean;
  logs: LogLine[];
  outputs: string[];
  lastError: string | null;
  failedNodeIds: string[];
  skippedNodeIds: string[];
  seed: (lastRun: LastRun | null) => void;
  start: (runId: string) => void;
  resume: (progress: RunProgress) => void;
  apply: (event: RunEvent) => void;
  clearLogs: () => void;
};

export const useRunStore = create<RunState>()((set) => ({
  ...emptyRunProgress(),
  running: false,
  logs: [],
  outputs: [],
  lastError: null,
  failedNodeIds: [],
  skippedNodeIds: [],

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
      failedNodeIds: [],
      skippedNodeIds: [],
      running: false,
      runId: null,
    });
  },

  start: (runId) =>
    set((s) =>
      s.runId === runId
        ? {}
        : {
            runId,
            running: true,
            outputs: [],
            lastError: null,
            failedNodeIds: [],
            skippedNodeIds: [],
          },
    ),

  resume: (progress) =>
    set((s) => ({
      ...resumeRunProgress(s, progress),
      running: true,
      outputs: [],
      lastError: null,
      failedNodeIds: [],
      skippedNodeIds: [],
    })),

  apply: (e) =>
    set((s) => {
      switch (e.type) {
        case "run:start":
          return {
            ...applyRunEvent(s, e),
            running: true,
            outputs: [],
            lastError: null,
            failedNodeIds: [],
            skippedNodeIds: [],
          };
        case "node:log":
          return {
            logs: [
              ...s.logs.slice(-1999),
              logLine({ at: e.at, level: e.level, nodeId: e.nodeId, message: e.message }),
            ],
          };
        case "node:error":
          return {
            ...applyRunEvent(s, e),
            logs: [
              ...s.logs,
              logLine({ at: Date.now(), level: "error", nodeId: e.nodeId, message: e.message }),
            ],
          };
        case "run:done":
          return {
            runId: e.runId,
            running: false,
            outputs: e.outputs,
            failedNodeIds: e.failedNodeIds ?? [],
            skippedNodeIds: e.skippedNodeIds ?? [],
            lastError: e.ok ? null : (e.error ?? (e.cancelled ? "Cancelled" : "Failed")),
          };
        default:
          return applyRunEvent(s, e);
      }
    }),

  clearLogs: () => set({ logs: [] }),
}));
