import { describe, expect, it } from "vitest";
import { nodeCard, nodeResult, workflowNode } from "../test-fixtures";
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
  const render = (id: string, params: Record<string, unknown>, disabled?: boolean) => ({
    ...workflowNode("output.render", params, id),
    ...(disabled ? { disabled } : {}),
  });

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

describe("Export video card", () => {
  const render = nodeResult({
    outputs: { media: { type: "media", src: null, kind: "video", durationMs: 15_400 } },
    artifacts: [
      { kind: "video", path: "a.mp4", bytes: 1_000_000 },
      { kind: "video", path: "b.mp4", bytes: 500_000 },
    ],
  });
  const texts = (variants: string[]) =>
    nodeCard("output.render", { variants }, { result: render }).map((r) => r.text);

  it("adds the length and file sizes after a run", () => {
    expect(texts(["16:9", "9:16"])).toEqual([
      "16:9 + 9:16",
      "MP4 · 1080p · 30 fps",
      "15.4 s",
      "1.4 MB",
    ]);
  });

  it("gives the short side of every shape it exports", () => {
    expect(texts(["16:9", "1:1"])[1]).toBe("MP4 · 1080p / 1920p · 30 fps");
  });
});
