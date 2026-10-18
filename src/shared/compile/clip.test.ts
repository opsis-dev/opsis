import { describe, expect, it } from "vitest";
import { groupOf, itemClip, sequenceClips, clipToSpec } from "./clip";
import { remapFrame } from "./remap";
import { DEFAULT_PROJECT_FONTS } from "../fonts/types";
import { flattenSpec, nodeSpans } from "../timeline/flatten";

const settings = {
  width: 1920,
  height: 1080,
  fps: 30 as const,
  theme: "dark" as const,
  fonts: DEFAULT_PROJECT_FONTS,
};

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

describe("held transitions", () => {
  const withSound = (id: string, frames: number) => ({
    ...itemClip(id, "core/title", {}, frames),
    audio: [{ id: `${id}:a`, src: "app:sfx/pop.wav", from: 0, volume: 1, role: "effect" as const }],
  });

  it("keep every clip whole and play the transition over held frames", () => {
    const seq = sequenceClips(
      "seq",
      [withSound("a", 45), withSound("b", 60), withSound("c", 30)],
      [
        undefined,
        { kind: "core/fade", durationInFrames: 30, hold: true },
        { kind: "core/fade", durationInFrames: 10, hold: true },
      ],
    );
    expect(seq.durationInFrames).toBe(45 + 30 + 60 + 10 + 30);
    const [a, b, c] = seq.tracks[0].items;
    expect([a.from, a.durationInFrames]).toEqual([0, 75]);
    expect([b.from, b.durationInFrames]).toEqual([45, 100]);
    expect([c.from, c.durationInFrames]).toEqual([135, 40]);
    const remapA = groupOf(a)?.remap ?? [];
    expect(remapFrame(remapA, 44)).toBe(44);
    expect(remapFrame(remapA, 74)).toBe(44);
    const remapB = groupOf(b)?.remap ?? [];
    expect(remapFrame(remapB, 10)).toBe(0);
    expect(remapFrame(remapB, 30)).toBe(0);
    expect(remapFrame(remapB, 89)).toBe(59);
    expect(remapFrame(remapB, 99)).toBe(59);
    expect(seq.audio.map((x) => x.from)).toEqual([0, 75, 145]);
  });

  it("delay a clip's own sound by the held frames instead of dropping it", () => {
    const video = (id: string, frames: number) =>
      itemClip(id, "core/video", { src: `project:${id}.mp4`, muted: false }, frames);
    const spec = (hold: boolean) =>
      clipToSpec(
        sequenceClips(
          "seq",
          [video("a", 45), video("b", 60)],
          [undefined, { kind: "core/fade", durationInFrames: 30, hold }],
        ),
        settings,
      );
    const sounds = (hold: boolean) =>
      flattenSpec(spec(hold)).audio.map((a) => [a.channel, a.from, a.to, a.trimBefore]);
    expect(sounds(false)).toEqual([
      ["a", 0, 45, 0],
      ["b", 15, 75, 0],
    ]);
    expect(sounds(true)).toEqual([
      ["a", 0, 45, 0],
      ["b", 75, 135, 0],
    ]);
  });

  it("drop a clip's own sound under a speed change", () => {
    const sped = clipToSpec(
      {
        ...itemClip("a", "core/video", { src: "project:a.mp4", muted: false }, 60),
        tracks: [
          {
            id: "t",
            items: [
              {
                id: "g",
                component: "core/group",
                from: 0,
                durationInFrames: 30,
                props: {
                  tracks: itemClip("a", "core/video", { src: "project:a.mp4", muted: false }, 60)
                    .tracks,
                  remap: [
                    [0, 0],
                    [30, 60],
                  ],
                },
              },
            ],
          },
        ],
        durationInFrames: 30,
      },
      settings,
    );
    const flat = flattenSpec(sped);
    expect(flat.bars.map((b) => [b.nodeId, b.hasSound])).toEqual([["a", true]]);
    expect(flat.audio).toEqual([]);
  });

  it("are not flagged on the timeline, while overlapping ones are", () => {
    const parts = [itemClip("a", "core/title", {}, 45), itemClip("b", "core/title", {}, 45)];
    const spec = (hold: boolean) =>
      clipToSpec(
        sequenceClips("seq", parts, [undefined, { kind: "core/fade", durationInFrames: 30, hold }]),
        settings,
      );
    expect(flattenSpec(spec(false)).transitions[0].covers).toBeCloseTo(30 / 45);
    expect(flattenSpec(spec(true)).transitions[0].covers).toBe(0);
  });
});
