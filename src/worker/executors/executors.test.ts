import { describe, expect, it } from "vitest";
import { NODE_DEFS } from "../../shared/nodes/registry";
import { getExecutor } from "./index";
import { atempoChain } from "./media";

describe("executors", () => {
  it("exist for every node that executes", () => {
    const missing = NODE_DEFS.filter((d) => d.executes && !getExecutor(d.type)).map((d) => d.type);
    expect(missing).toEqual([]);
  });
});

describe("atempoChain", () => {
  it("splits fast rates into 2× stages", () => {
    expect(atempoChain(64)).toBe(Array.from({ length: 6 }, () => "atempo=2.0000").join(","));
  });

  it("splits slow rates into 0.5× stages", () => {
    expect(atempoChain(0.25)).toBe("atempo=0.5000,atempo=0.5000");
  });

  it("ends with the remainder", () => {
    expect(atempoChain(3)).toBe("atempo=2.0000,atempo=1.5000");
  });
});
