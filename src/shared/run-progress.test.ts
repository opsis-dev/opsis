import { describe, expect, it } from "vitest";
import type { RunEvent } from "./schema/events";
import type { NodeResult } from "./schema/values";
import {
  applyRunEvent,
  emptyRunProgress,
  exportedArtifacts,
  failureMessage,
  resumeRunProgress,
} from "./run-progress";

const result = (finishedAt: number, paths: string[] = []): NodeResult => ({
  hash: "h",
  outputs: {},
  artifacts: paths.map((path) => ({ kind: "video", path })),
  finishedAt,
  durationMs: 1,
});

const run = (events: RunEvent[]) => events.reduce(applyRunEvent, emptyRunProgress());

describe("applyRunEvent", () => {
  it("follows a run from start to finish", () => {
    const p = run([
      { type: "run:start", runId: "r", nodeIds: ["a", "b", "c", "d"], at: 0 },
      { type: "node:done", runId: "r", nodeId: "a", cached: true, result: result(1, ["a.mp4"]) },
      { type: "node:start", runId: "r", nodeId: "b", at: 1 },
      { type: "node:progress", runId: "r", nodeId: "b", progress: 0.4, message: "Recording" },
      { type: "node:error", runId: "r", nodeId: "c", message: "boom", traceId: "t" },
      { type: "node:skipped", runId: "r", nodeId: "d", reason: "Something before it failed" },
    ]);
    expect(p.runId).toBe("r");
    expect(p.statuses).toEqual({ a: "cached", b: "running", c: "error", d: "skipped" });
    expect(p.progress).toEqual({ a: { value: 1 }, b: { value: 0.4, message: "Recording" } });
    expect(p.errors).toEqual({ c: "boom" });
    expect(p.traces).toEqual({ c: "t" });
    expect(Object.keys(p.results)).toEqual(["a"]);
  });

  it("clears only the nodes a new run includes", () => {
    const first = run([
      { type: "run:start", runId: "r1", nodeIds: ["a", "b"], at: 0 },
      { type: "node:done", runId: "r1", nodeId: "a", cached: false, result: result(1) },
      { type: "node:error", runId: "r1", nodeId: "b", message: "boom" },
    ]);
    const second = applyRunEvent(first, {
      type: "run:start",
      runId: "r2",
      nodeIds: ["a"],
      at: 2,
    });
    expect(second.statuses).toEqual({ a: "queued", b: "error" });
    expect(second.errors).toEqual({ b: "boom" });
    expect(second.progress).toEqual({});
  });

  it("resumes over a freshly opened project", () => {
    const opened = { ...emptyRunProgress(), errors: { old: "earlier" }, results: { x: result(1) } };
    const saved = run([
      { type: "run:start", runId: "r", nodeIds: ["a", "b"], at: 0 },
      { type: "node:done", runId: "r", nodeId: "a", cached: true, result: result(2, ["a.mp4"]) },
      { type: "node:progress", runId: "r", nodeId: "b", progress: 0.5 },
    ]);
    const resumed = resumeRunProgress(opened, saved);
    expect(resumed.runId).toBe("r");
    expect(resumed.statuses).toEqual({ a: "cached", b: "queued" });
    expect(resumed.progress.b).toEqual({ value: 0.5 });
    expect(resumed.errors).toEqual({ old: "earlier" });
    expect(Object.keys(resumed.results).toSorted()).toEqual(["a", "x"]);
  });
});

describe("failureMessage", () => {
  it("names the failed nodes and counts skips apart", () => {
    expect(failureMessage(["Export film"], 0)).toBe("Export film failed");
    expect(failureMessage(["Export film", "GIF"], 3)).toBe(
      "2 nodes failed: Export film, GIF, 3 skipped",
    );
    expect(failureMessage([], 1)).toBe("1 node was skipped");
  });
});

describe("exportedArtifacts", () => {
  it("lists every export's files, newest first", () => {
    const nodes = [
      { id: "film", type: "output.render" },
      { id: "code", type: "output.code" },
      { id: "rec", type: "capture.record" },
    ];
    const files = exportedArtifacts(nodes, {
      film: result(1, ["film.mp4"]),
      code: result(5, ["code"]),
      rec: result(9, ["rec.mp4"]),
    });
    expect(files.map((a) => a.path)).toEqual(["code", "film.mp4"]);
  });
});
