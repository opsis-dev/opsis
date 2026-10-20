import { describe, expect, it } from "vitest";
import { emptyClip, sceneItemCount } from "../compile/clip";
import { MAX_FRAMES } from "../limits";
import type { ClipValue } from "../schema/values";
import { compileClip, compileNode, nodeCard } from "../test-fixtures";
import { branchPort } from "./timeline";

const compileOver = (type: string, params: Record<string, unknown>): ClipValue =>
  compileClip(type, params, { clip: emptyClip(90) });

const startOf = (clip: ClipValue) => clip.tracks[0].items[0].from;

describe("timeline node lengths", () => {
  it("rounds a delay to whole frames, down to none for less than half a frame", () => {
    expect(startOf(compileOver("timeline.delay", { ms: 10 }))).toBe(0);
    expect(startOf(compileOver("timeline.delay", { ms: 0 }))).toBe(0);
    expect(startOf(compileOver("timeline.delay", { ms: 500 }))).toBe(15);
  });

  it("rounds a trim start to whole frames, down to none for less than half a frame", () => {
    const startFrom = (startMs: number) =>
      compileOver("timeline.trim", { startMs }).tracks[0].items[0].props.startFrom;
    expect(startFrom(10)).toBe(0);
    expect(startFrom(1000)).toBe(30);
  });
});

describe("timeline node cards", () => {
  it("adds a quiet note on Speed from 4× up", () => {
    const notes = (rate: number) =>
      nodeCard("timeline.speed", { rate }).filter((r) => r.tone === "muted");
    expect(notes(2)).toEqual([]);
    expect(notes(4)).toHaveLength(1);
  });

  it("counts Sequence clips and cuts with their own transition and warns about the worst one", () => {
    const cuts = [
      { before: "in3", transition: "core/wipe", transitionDuration: 50, keepWhole: false },
      { before: "in5", transition: "core/iris", transitionDuration: 10, keepWhole: false },
    ];
    const context = {
      connected: ["in1", "in2", "in3"],
      inputFrames: { in1: 300, in2: 100, in3: 300 },
      clipFrames: 690,
    };
    expect(
      nodeCard("timeline.sequence", { transitionDuration: 10, cuts }, context).map((r) => r.text),
    ).toEqual([
      "3 clips · 23 s",
      "Fade · 0.33 s",
      "1 cut uses its own transition",
      "Covers 50% of a 3.33 s clip",
    ]);
    const held = nodeCard(
      "timeline.sequence",
      { transitionDuration: 10, cuts: [{ ...cuts[0], keepWhole: true }] },
      context,
    );
    expect(held.some((r) => r.text.startsWith("Covers"))).toBe(false);
  });
});

describe("Layers", () => {
  const stacked = (params: Record<string, unknown>) =>
    compileClip("timeline.stack", params, { in1: emptyClip(30), in2: emptyClip(30) });

  it("marks its layers so a sequencer can land them one by one", () => {
    expect(sceneItemCount(stacked({}))).toBe(2);
  });

  it("stays one item when it keeps its layout in other shapes", () => {
    expect(sceneItemCount(stacked({ keepLayout: true }))).toBe(1);
  });
});

describe("Repeat", () => {
  it("refuses to make a film longer than the frame limit", () => {
    expect(() =>
      compileClip("timeline.loop", { times: 100 }, { clip: emptyClip(MAX_FRAMES / 50) }),
    ).toThrow("Repeated 100 times the clip would run");
    expect(
      compileClip("timeline.loop", { times: 50 }, { clip: emptyClip(MAX_FRAMES / 50) })
        .durationInFrames,
    ).toBe(MAX_FRAMES);
  });
});

describe("Choose by variable", () => {
  it("uses the first clip when the variable has a value, or the given one", () => {
    expect(branchPort("A", "A")).toBe("a");
    expect(branchPort(" A ", "A")).toBe("a");
    expect(branchPort("B", "A")).toBe("b");
    expect(branchPort("B", "")).toBe("a");
    expect(branchPort(undefined, "")).toBe("b");
    expect(branchPort("  ", "")).toBe("b");
    expect(branchPort(undefined, "A")).toBe("b");
  });

  it("reads a variable name typed with spaces around it", () => {
    const pick = compileNode(
      "timeline.branch",
      { variable: " Version ", equals: "A" },
      { a: emptyClip(10), b: emptyClip(20) },
      undefined,
      { Version: "A" },
    ).clip;
    expect(pick?.type === "clip" ? pick.durationInFrames : null).toBe(10);
  });

  it("asks for the clip it chose when that input is empty", () => {
    expect(() =>
      compileNode("timeline.branch", { variable: "Version", equals: "" }, { a: emptyClip(10) }),
    ).toThrow("Connect a clip to “Otherwise”.");
  });

  it("says what it checks on the card", () => {
    expect(nodeCard("timeline.branch", { equals: "" })[0].text).toBe("Version has a value");
    expect(nodeCard("timeline.branch", { equals: "A" })[0].text).toBe("Version = “A”");
  });
});
