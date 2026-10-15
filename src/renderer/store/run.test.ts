import { describe, expect, it } from "vitest";
import type { NodeResult } from "@/shared/schema/values";
import { useRunStore } from "./run";

const result = (finishedAt: number, traceId: string): NodeResult => ({
  hash: "h",
  outputs: {},
  artifacts: [],
  finishedAt,
  durationMs: 1,
  traceId,
});

describe("run store seed", () => {
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
    useRunStore.getState().apply({
      type: "node:error",
      runId: "r",
      nodeId: "rec",
      message: "boom",
      traceId: "failed",
    });
    expect(useRunStore.getState().traces.rec).toBe("failed");
  });
});
