import { describe, expect, it } from "vitest";
import { compileNode, nodeCard, nodeResult, recordResult } from "../test-fixtures";
import { loginOnceNode, stillNode } from "./capture";
import { parseNodeParams } from "./define";
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

describe("Sign in once settings", () => {
  const problem = (params: Record<string, unknown>) => {
    const parsed = parseParams(loginOnceNode.params, params);
    return parsed.ok ? null : parsed.problem;
  };

  it("asks for the sign-in page and, when finishing on an element, the element", () => {
    expect(problem({ url: " " })).toBe("Add the sign-in page");
    expect(problem({ url: "https://example.com/login", doneWhen: "selector", selector: "" })).toBe(
      "Add the element that appears after signing in",
    );
    expect(
      problem({ url: "https://example.com/login", doneWhen: "selector", selector: ".avatar" }),
    ).toBeNull();
  });

  it("fills the site address into the sign-in page", () => {
    const parsed = parseNodeParams(
      loginOnceNode,
      { params: {} },
      { "Site address": "https://app.example.com" },
    );
    expect(parsed.url).toBe("https://app.example.com/login");
  });
});

describe("Screenshot clip", () => {
  type ShotImage = { src: string; width?: number; height?: number; scale?: number };
  const compileStill = (params: Record<string, unknown>, image?: ShotImage) => {
    const out = compileNode(
      "capture.still",
      params,
      {},
      image ? nodeResult({ outputs: { image: { type: "image", ...image } } }) : undefined,
    );
    const clip = out.clip;
    if (clip?.type !== "clip") throw new Error("no clip");
    return { image: out.image, item: clip.tracks[0].items[0], frames: clip.durationInFrames };
  };

  it("shows a placeholder until the screenshot is taken", () => {
    const { image, item, frames } = compileStill({ durationMs: 3000 });
    expect(image).toEqual({ type: "image", src: null });
    expect(item.component).toBe("core/placeholder");
    expect(frames).toBe(90);
  });

  it("shows the screenshot with its size once taken", () => {
    const { item } = compileStill(
      { look: "cutout" },
      { src: "project:shot.png", width: 400, height: 300, scale: 2 },
    );
    expect(item.component).toBe("core/still");
    expect(item.props).toMatchObject({
      src: "project:shot.png",
      look: "cutout",
      shot: { width: 400, height: 300, scale: 2 },
    });
  });

  it("still draws a screenshot whose size isn't known", () => {
    const { item } = compileStill({}, { src: "project:shot.png" });
    expect(item.component).toBe("core/still");
    expect(item.props).toMatchObject({ src: "project:shot.png", shot: undefined });
  });

  it("never lasts no time at all", () => {
    expect(parseParams(stillNode.params, { durationMs: 0 })).toEqual({
      ok: false,
      problem: "Length is too small",
    });
  });
});
