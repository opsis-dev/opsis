import type { NodeResult } from "./values";

export type NodeRunStatus = "idle" | "queued" | "running" | "cached" | "done" | "error" | "skipped";

export type LogLevel = "debug" | "info" | "warn" | "error";

export type RunEvent =
  | { type: "run:start"; runId: string; nodeIds: string[]; at: number }
  | { type: "node:start"; runId: string; nodeId: string; at: number }
  | { type: "node:progress"; runId: string; nodeId: string; progress: number; message?: string }
  | {
      type: "node:log";
      runId: string;
      nodeId?: string;
      level: LogLevel;
      message: string;
      at: number;
    }
  | { type: "node:done"; runId: string; nodeId: string; cached: boolean; result: NodeResult }
  | {
      type: "node:error";
      runId: string;
      nodeId: string;
      message: string;
      stack?: string;
      traceId?: string;
    }
  | { type: "node:skipped"; runId: string; nodeId: string; reason: string }
  | {
      type: "run:done";
      runId: string;
      ok: boolean;
      cancelled: boolean;
      error?: string;
      outputs: string[];
      at: number;
    };
