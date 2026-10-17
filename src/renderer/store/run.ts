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

export const MAX_LOG_LINES = 2000;

const withLine = (logs: LogLine[], line: LogLine): LogLine[] => [
  ...logs.slice(-(MAX_LOG_LINES - 1)),
  line,
];

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

const freshRun = () => ({
  running: true,
  outputs: [],
  lastError: null,
  failedNodeIds: [],
  skippedNodeIds: [],
});

const newerFailures = (lastRun: LastRun | null) =>
  Object.entries(lastRun?.failures ?? {}).filter(
    ([id, f]) => f.at > (lastRun?.nodes[id]?.finishedAt ?? 0),
  );

function traceIds(lastRun: LastRun | null): Record<string, string> {
  const traces: Record<string, string> = {};
  for (const [id, r] of Object.entries(lastRun?.nodes ?? {})) if (r.traceId) traces[id] = r.traceId;
  for (const [id, f] of newerFailures(lastRun)) if (f.traceId) traces[id] = f.traceId;
  return traces;
}

const errorsOf = (lastRun: LastRun | null): Record<string, string> =>
  Object.fromEntries(newerFailures(lastRun).map(([id, f]) => [id, f.message]));

export const useRunStore = create<RunState>()((set) => ({
  ...emptyRunProgress(),
  ...freshRun(),
  running: false,
  logs: [],

  seed: (lastRun) =>
    set({
      ...emptyRunProgress(),
      ...freshRun(),
      traces: traceIds(lastRun),
      results: lastRun?.nodes ?? {},
      errors: errorsOf(lastRun),
      logs: [],
      running: false,
    }),

  start: (runId) => set((s) => (s.runId === runId ? {} : { ...freshRun(), runId })),

  resume: (progress) => set((s) => ({ ...resumeRunProgress(s, progress), ...freshRun() })),

  apply: (e) =>
    set((s) => {
      switch (e.type) {
        case "run:start":
          return { ...applyRunEvent(s, e), ...freshRun() };
        case "node:log":
          return {
            logs: withLine(
              s.logs,
              logLine({ at: e.at, level: e.level, nodeId: e.nodeId, message: e.message }),
            ),
          };
        case "node:error":
          return {
            ...applyRunEvent(s, e),
            logs: withLine(
              s.logs,
              logLine({ at: Date.now(), level: "error", nodeId: e.nodeId, message: e.message }),
            ),
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
