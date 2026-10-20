import { describe, expect, it } from "vitest";
import { NODE_DEFS } from "../../shared/nodes/registry";
import { EXECUTORS, executorTable, getExecutor } from "./index";

describe("executors", () => {
  it("exist for every node that executes", () => {
    const missing = NODE_DEFS.filter((d) => d.executes && !getExecutor(d.type)).map((d) => d.type);
    expect(missing).toEqual([]);
  });

  it("are each for a node that executes, with no node twice", () => {
    const executing = new Set(NODE_DEFS.filter((d) => d.executes).map((d) => d.type));
    expect(EXECUTORS.map((e) => e.type).filter((t) => !executing.has(t))).toEqual([]);
    expect(new Set(EXECUTORS.map((e) => e.type)).size).toBe(EXECUTORS.length);
  });

  it("refuse a second executor for the same node", () => {
    expect(() => executorTable([EXECUTORS[0], EXECUTORS[0]])).toThrow(
      `Two executors for ${EXECUTORS[0].type}`,
    );
  });
});
