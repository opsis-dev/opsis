import { describe, expect, it } from "vitest";
import { emptyClip, sceneItemCount } from "../compile/clip";
import type { ClipValue } from "../schema/values";
import { compileNode, nodeCard } from "../test-fixtures";

function compileOver(type: string, params: Record<string, unknown>): ClipValue {
  const clip = compileNode(type, params, { clip: emptyClip(90) }).clip;
  if (clip?.type !== "clip") throw new Error(`no clip from ${type}`);
  return clip;
}

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
  const stacked = (params: Record<string, unknown>) => {
    const clip = compileNode("timeline.stack", params, {
      in1: emptyClip(30),
      in2: emptyClip(30),
    }).clip;
    if (clip?.type !== "clip") throw new Error("no clip from Layers");
    return clip;
  };

  it("marks its layers so a sequencer can land them one by one", () => {
    expect(sceneItemCount(stacked({}))).toBe(2);
  });

  it("stays one item when it keeps its layout in other shapes", () => {
    expect(sceneItemCount(stacked({ keepLayout: true }))).toBe(1);
  });
});
