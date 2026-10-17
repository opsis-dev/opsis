import { describe, expect, it } from "vitest";
import { NODE_DEFS } from "../../shared/nodes/registry";
import { getExecutor } from "./index";

describe("executors", () => {
  it("exist for every node that executes", () => {
    const missing = NODE_DEFS.filter((d) => d.executes && !getExecutor(d.type)).map((d) => d.type);
    expect(missing).toEqual([]);
  });
});
