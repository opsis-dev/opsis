import { describe, expect, it } from "vitest";
import {
  itemClip,
  nodeSpans,
  remapDuration,
  remapFrame,
  remapFromSegments,
  sequenceClips,
  clipToSpec,
} from "./clip";

const settings = { width: 1920, height: 1080, fps: 30 as const, theme: "dark" as const };

describe("remaps", () => {
  it("maps output frames to inner frames piecewise-linearly", () => {
    const remap: Array<[number, number]> = [
      [0, 0],
      [10, 40],
      [20, 50],
    ];
    expect(remapFrame(remap, 5)).toBe(20);
    expect(remapFrame(remap, 15)).toBe(45);
    expect(remapFrame(remap, 25)).toBe(55);
  });

  it("builds a remap from speed segments", () => {
    const remap = remapFromSegments(100, [[20, 60, 4]]);
    expect(remapDuration(remap)).toBe(70);
    expect(remapFrame(remap, 30)).toBe(60);
  });
});

describe("sequenceClips", () => {
  it("clamps a transition to the shorter clip", () => {
    const a = itemClip("a", "core/solid", {}, 10);
    const b = itemClip("b", "core/solid", {}, 100);
    const seq = sequenceClips(
      "s",
      [a, b],
      [undefined, { kind: "core/fade", durationInFrames: 30 }],
    );
    expect(seq.tracks[0].items[1].enter?.durationInFrames).toBe(9);
    expect(seq.durationInFrames).toBe(101);
  });

  it("records absolute spans per node", () => {
    const a = itemClip("a", "core/solid", {}, 30);
    const b = itemClip("b", "core/solid", {}, 30);
    const spec = clipToSpec(sequenceClips("s", [a, b]), settings);
    expect(nodeSpans(spec)).toEqual({ a: { from: 0, to: 30 }, b: { from: 30, to: 60 } });
  });
});
