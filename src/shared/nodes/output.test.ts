import { describe, expect, it } from "vitest";
import type { WorkflowNode } from "../schema/workflow";
import { exportReframe, variantCanvas, variantSize } from "./output";

describe("variantSize", () => {
  it("keeps the long edge and even dimensions", () => {
    const s = { width: 1920, height: 1080 };
    expect(variantSize("16:9", s)).toEqual({ width: 1920, height: 1080 });
    expect(variantSize("9:16", s)).toEqual({ width: 1080, height: 1920 });
    expect(variantSize("1:1", s)).toEqual({ width: 1920, height: 1920 });
    expect(variantSize("4:5", s)).toEqual({ width: 1536, height: 1920 });
  });
});

describe("variantCanvas", () => {
  it("keeps the short edge", () => {
    expect(variantCanvas("16:9", 1080)).toEqual({ width: 1920, height: 1080 });
    expect(variantCanvas("1:1", 1080)).toEqual({ width: 1080, height: 1080 });
    expect(variantCanvas("4:5", 1080)).toEqual({ width: 1080, height: 1350 });
    expect(variantCanvas("9:16", 1080)).toEqual({ width: 1080, height: 1920 });
  });
});

describe("exportReframe", () => {
  const render = (id: string, params: Record<string, unknown>, disabled?: boolean) =>
    ({
      id,
      type: "output.render",
      position: { x: 0, y: 0 },
      params,
      ...(disabled ? { disabled } : {}),
    }) satisfies WorkflowNode;

  it("uses the first enabled Export video node", () => {
    expect(
      exportReframe([render("a", { reframe: "fit" }, true), render("b", { reframe: "center" })]),
    ).toBe("center");
  });

  it("follows the action without an export or with a setting it doesn't know", () => {
    expect(exportReframe([])).toBe("follow");
    expect(exportReframe([render("a", { reframe: "sideways" })])).toBe("follow");
    expect(exportReframe([render("a", {})])).toBe("follow");
  });
});
