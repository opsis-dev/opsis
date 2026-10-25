import { CANCELLED } from "@opsis/core/errors";
import { appendCapped } from "@opsis/core/lists";
import { pick } from "@opsis/core/objects";
import {
  applyRunEvent,
  emptyRunProgress,
  resumeRunProgress,
  type RunProgress,
} from "@opsis/core/run-progress";
import type { LogLevel, RunEvent } from "@opsis/core/schema/events";
import type { LastRun } from "@opsis/core/schema/values";
import { create } from "zustand";

export type LogLine = {
  id: number;
  at: number;
  level: LogLevel;
  nodeId?: string;
  message: string;
};

let logCount = 0;

type NodeLog = Extract<RunEvent, { type: "node:log" }>;

type NodeError = Extract<RunEvent, { type: "node:error" }>;

const makeLine = (fields: Omit<LogLine, "id">): LogLine => ({ id: ++logCount, ...fields });

const logLine = ({ at, level, nodeId, message }: NodeLog): LogLine =>
  makeLine({ at, level, nodeId, message });

const errorLine = ({ nodeId, message }: NodeError): LogLine =>
  makeLine({ at: Date.now(), level: "error", nodeId, message });

export const MAX_LOG_LINES = 2000;

const withLine = (logs: LogLine[], line: LogLine): LogLine[] =>
  appendCapped(logs, line, MAX_LOG_LINES);

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
  forgetErrors: (ids: readonly string[]) => void;
};

const freshRun = (running: boolean) => ({
  running,
  outputs: [],
  lastError: null,
  failedNodeIds: [],
  skippedNodeIds: [],
});

const idleRun = () => ({ ...emptyRunProgress(), ...freshRun(false), logs: [] });

const RUN_FAILED = "Failed";

type RunDone = Extract<RunEvent, { type: "run:done" }>;

const runDoneError = (e: RunDone): string | null => {
  if (e.ok) return null;
  return e.error ?? (e.cancelled ? CANCELLED : RUN_FAILED);
};

const finishedRun = (e: RunDone): Partial<RunState> => ({
  runId: e.runId,
  running: false,
  outputs: e.outputs,
  failedNodeIds: e.failedNodeIds ?? [],
  skippedNodeIds: e.skippedNodeIds ?? [],
  lastError: runDoneError(e),
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
  ...idleRun(),

  seed: (lastRun) =>
    set({
      ...idleRun(),
      traces: traceIds(lastRun),
      results: lastRun?.nodes ?? {},
      errors: errorsOf(lastRun),
    }),

  start: (runId) => set((s) => (s.runId === runId ? {} : { ...freshRun(true), runId })),

  resume: (progress) => set((s) => ({ ...resumeRunProgress(s, progress), ...freshRun(true) })),

  apply: (e) =>
    set((s) => {
      switch (e.type) {
        case "run:start":
          return { ...applyRunEvent(s, e), ...freshRun(true) };
        case "node:log":
          return { logs: withLine(s.logs, logLine(e)) };
        case "node:error":
          return { ...applyRunEvent(s, e), logs: withLine(s.logs, errorLine(e)) };
        case "run:done":
          return finishedRun(e);
        default:
          return applyRunEvent(s, e);
      }
    }),

  clearLogs: () => set({ logs: [] }),

  forgetErrors: (ids) =>
    set((s) =>
      ids.some((id) => s.errors[id] !== undefined)
        ? {
            errors: Object.fromEntries(
              Object.entries(s.errors).filter(([id]) => !ids.includes(id)),
            ),
          }
        : {},
    ),
}));

const RUN_SYNC_KEYS = [
  "runId",
  "running",
  "statuses",
  "progress",
  "cached",
  "errors",
  "traces",
  "logs",
  "results",
  "outputs",
  "lastError",
  "failedNodeIds",
  "skippedNodeIds",
] as const;

export type RunSnapshot = Pick<RunState, (typeof RUN_SYNC_KEYS)[number]>;

export const runSnapshot = (): RunSnapshot => pick(useRunStore.getState(), RUN_SYNC_KEYS);
