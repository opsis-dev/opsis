import { describe, expect, it } from "vitest";
import { nodeCard, nodeResult, recordResult } from "../test-fixtures";
import { stillNode } from "./capture";
import { parseParams } from "./params";

describe("capture cards", () => {
  it("adds the take's length and actions to Record take", () => {
    const record = recordResult({
      viewport: { width: 1920, height: 1080 },
      dpr: 1,
      t0: 0,
      durationMs: 14_500,
      actions: [
        { id: "a", kind: "click", t: 1000 },
        { id: "b", kind: "click", t: 2000 },
      ],
    });
    expect(nodeCard("capture.record", {}, { result: record }).map((r) => r.text)).toEqual([
      "30 fps",
      "Smooth cursor",
      "14.5 s",
      "2 actions",
    ]);
  });

  it("tells whether a sign-in is saved", () => {
    expect(
      nodeCard("capture.loginOnce", { secretName: "sauce demo" }, { secrets: ["sauce-demo"] }).at(
        -1,
      ),
    ).toMatchObject({ text: "Signed in" });
    expect(
      nodeCard("capture.loginOnce", { secretName: "other" }, { secrets: [] }).at(-1),
    ).toMatchObject({ text: "Not signed in yet", tone: "warn" });
  });

  it("describes the element a Screenshot captures in words", () => {
    const first = (params: Record<string, unknown>) => nodeCard("capture.still", params)[0];
    expect(first({ region: "full-page" })).toMatchObject({ text: "Whole page" });
    expect(first({ region: "element", target: "Pricing", targetRole: "heading" })).toMatchObject({
      text: "“Pricing” heading",
    });
    expect(first({ region: "element", target: "" })).toMatchObject({
      text: "Pick an element",
      tone: "warn",
    });
  });

  it("shows a Screenshot's size only when both sides are known", () => {
    const sized = (image: Record<string, unknown>) =>
      nodeCard(
        "capture.still",
        {},
        { result: nodeResult({ outputs: { image: { type: "image", src: null, ...image } } }) },
      ).map((r) => r.text);
    expect(sized({ width: 1200, height: 800 })).toContain("1200 × 800");
    expect(sized({ width: 1200 }).some((t) => t.includes("×"))).toBe(false);
  });
});

describe("Screenshot settings", () => {
  it("asks for the element when capturing one element", () => {
    expect(parseParams(stillNode.params, { region: "element", target: "" })).toEqual({
      ok: false,
      problem: "Pick an element",
    });
    expect(parseParams(stillNode.params, { region: "element", target: "Go" }).ok).toBe(true);
    expect(parseParams(stillNode.params, { region: "viewport" }).ok).toBe(true);
  });
});
