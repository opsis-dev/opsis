import { beforeEach, describe, expect, it } from "vitest";
import type { RunEvent } from "@/generated/bindings";
import { nodeResult } from "@/test/graph";
import { MAX_LOG_LINES, useRunStore } from "./run";

const result = (finishedAt: number, traceId: string) => nodeResult({ finishedAt, traceId });

type RunDone = Extract<RunEvent, { type: "run:done" }>;

const runDone = (runId: string, overrides: Partial<RunDone> = {}): RunDone => ({
  type: "run:done",
  runId,
  ok: true,
  cancelled: false,
  outputs: [],
  at: 0,
  ...overrides,
});

const nodeDone = (
  runId: string,
  nodeId: string,
  finishedAt: number,
  traceId: string,
): RunEvent => ({
  type: "node:done",
  runId,
  nodeId,
  cached: false,
  result: result(finishedAt, traceId),
});

const nodeError = (runId: string, nodeId: string, message: string, traceId?: string): RunEvent => ({
  type: "node:error",
  runId,
  nodeId,
  message,
  ...(traceId ? { traceId } : {}),
});

describe("run store", () => {
  beforeEach(() => useRunStore.getState().seed(null));

  it("opens the newest trace of each node, failed or not", () => {
    useRunStore.getState().seed({
      updatedAt: 300,
      nodes: { rec: result(100, "ok-old"), still: result(250, "ok-new") },
      failures: {
        rec: { message: "Assertion failed", at: 200, traceId: "failed-new" },
        still: { message: "Timeout", at: 150, traceId: "failed-old" },
        login: { message: "Closed", at: 120 },
      },
    });
    expect(useRunStore.getState().traces).toEqual({ rec: "failed-new", still: "ok-new" });
    expect(useRunStore.getState().errors).toEqual({ rec: "Assertion failed", login: "Closed" });
  });

  it("points the trace at the failed take when a node fails during a run", () => {
    useRunStore.getState().seed({ updatedAt: 1, nodes: { rec: result(1, "ok") } });
    useRunStore.getState().apply(nodeError("r", "rec", "boom", "failed"));
    expect(useRunStore.getState().traces.rec).toBe("failed");
  });

  it("keeps finished nodes when the editor reopens during a run", () => {
    const store = useRunStore.getState();
    store.seed({ updatedAt: 1, nodes: {} });
    store.resume({
      runId: "r",
      statuses: { rec: "done", film: "running" },
      progress: { rec: { value: 1 }, film: { value: 0.3 } },
      cached: { rec: false },
      errors: {},
      traces: {},
      results: { rec: result(5, "t") },
    });
    const s = useRunStore.getState();
    expect(s.running).toBe(true);
    expect(s.statuses).toEqual({ rec: "done", film: "running" });
    expect(s.progress.film?.value).toBe(0.3);
    expect(s.results.rec?.traceId).toBe("t");
  });

  it("keeps an earlier failure when a later run leaves that node out", () => {
    const store = useRunStore.getState();
    store.apply({ type: "run:start", runId: "r1", nodeIds: ["a", "b", "c"], at: 0 });
    store.apply(nodeDone("r1", "a", 1, "x"));
    store.apply(nodeError("r1", "b", "boom"));
    store.apply({
      type: "node:skipped",
      runId: "r1",
      nodeId: "c",
      reason: "Something before it failed",
    });
    store.apply(
      runDone("r1", {
        ok: false,
        error: "B failed, 1 skipped",
        failedNodeIds: ["b"],
        skippedNodeIds: ["c"],
        at: 2,
      }),
    );
    expect(useRunStore.getState().failedNodeIds).toEqual(["b"]);
    store.start("r2");
    store.apply({ type: "run:start", runId: "r2", nodeIds: ["c"], at: 3 });
    store.apply(nodeDone("r2", "c", 4, "y"));
    store.apply(runDone("r2", { at: 5 }));
    const s = useRunStore.getState();
    expect(s.errors.b).toBe("boom");
    expect(s.statuses.b).toBe("error");
    expect(s.statuses.c).toBe("done");
    expect(s.running).toBe(false);
  });

  it("doesn't restart a run whose events came before its id", () => {
    const store = useRunStore.getState();
    store.apply({ type: "run:start", runId: "fast", nodeIds: ["a"], at: 0 });
    store.apply(runDone("fast", { at: 1 }));
    store.start("fast");
    expect(useRunStore.getState().running).toBe(false);
  });

  it("keeps the log at its limit however lines arrive", () => {
    const store = useRunStore.getState();
    store.apply({ type: "run:start", runId: "r", nodeIds: ["a"], at: 0 });
    for (let i = 0; i < MAX_LOG_LINES; i++) {
      store.apply({
        type: "node:log",
        runId: "r",
        nodeId: "a",
        level: "info",
        message: `log ${i}`,
        at: i,
      });
      store.apply(nodeError("r", "a", `error ${i}`));
    }
    const logs = useRunStore.getState().logs;
    expect(logs).toHaveLength(MAX_LOG_LINES);
    expect(logs.at(-1)?.message).toBe(`error ${MAX_LOG_LINES - 1}`);
  });
});
