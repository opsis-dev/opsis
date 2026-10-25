import { describe, expect, it } from "vitest";
import { DEFAULT_PROJECT_FONTS } from "../fonts/types";
import type { AudioItem, Remap, TransitionRef } from "../schema/timeline";
import { must } from "../test-fixtures";
import { flattenSpec, nodeSpans } from "../timeline/flatten";
import {
  clipToSpec,
  emptyClip,
  groupItem,
  groupOf,
  itemClip,
  markLayers,
  recordingClip,
  recordingOf,
  sceneItemCount,
  sceneSequence,
  sequenceClips,
  stackClips,
  wrapClip,
  wrapSounds,
} from "./clip";
import { defaultRecordingProps } from "./recording";
import { remapFrame } from "./remap";

const popSound = (id: string) => ({
  id: `${id}:a`,
  src: "app:sfx/pop.wav",
  from: 0,
  volume: 1,
  role: "effect" as const,
});

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
    audio: [popSound(id)],
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

describe("speed changes", () => {
  it("drop a clip's own sound", () => {
    const video = itemClip("a", "core/video", { src: "project:a.mp4", muted: false }, 60);
    const remap: Remap = [
      [0, 0],
      [30, 60],
    ];
    const group = groupItem(
      { id: "g", from: 0, durationInFrames: 30 },
      { tracks: video.tracks, remap },
    );
    const flat = flattenSpec(
      clipToSpec(
        { ...video, tracks: [{ id: "t", items: [group] }], durationInFrames: 30 },
        settings,
      ),
    );
    expect(flat.bars.map((b) => [b.nodeId, b.hasSound])).toEqual([["a", true]]);
    expect(flat.audio).toEqual([]);
  });
});

describe("sceneSequence", () => {
  const sequencer = { component: "remocn/spring-settle", props: {}, backgrounds: [null, null] };
  const solid = (id: string, frames: number) => itemClip(id, "core/solid", {}, frames);
  const layered = (offsets: number[], front = solid("y", 30)) =>
    markLayers(
      stackClips("l", [solid("x", 30), front], offsets, {
        ports: ["in1", "in2"],
        move: (index) => ({ nodeId: "l", param: "offsetsMs", unit: "ms", index }),
      }),
    );
  const scenesOf = (clip: ReturnType<typeof sceneSequence>) =>
    must(groupOf(clip.tracks[0].items[0]), "sequencer group").tracks[0].items;

  it("plays scenes one after another with the pause between them, and no pause after the last", () => {
    const clip = sceneSequence(
      "s",
      [
        { port: "in1", clip: solid("a", 30) },
        { port: "in3", clip: solid("b", 45) },
      ],
      { gap: 5, sequencer },
    );
    expect(clip.durationInFrames).toBe(80);
    expect(groupOf(clip.tracks[0].items[0])?.sequencer).toEqual(sequencer);
    expect(scenesOf(clip).map((s) => [s.from, s.durationInFrames, s.slot])).toEqual([
      [0, 30, { nodeId: "s", port: "in1" }],
      [35, 45, { nodeId: "s", port: "in3" }],
    ]);
    expect(nodeSpans(clipToSpec(clip, settings))).toMatchObject({
      a: { from: 0, to: 30 },
      b: { from: 35, to: 80 },
    });
  });

  it("makes a plain clip one item and each layer of a Layers clip its own item", () => {
    const layers = layered([0, 12]);
    expect(sceneItemCount(solid("p", 30))).toBe(1);
    expect(sceneItemCount(layers)).toBe(2);
    expect(sceneItemCount(wrapClip(layers, "w"))).toBe(1);
    const clip = sceneSequence(
      "s",
      [
        { port: "in1", clip: solid("p", 30) },
        { port: "in2", clip: layers },
      ],
      { gap: 0, sequencer },
    );
    const [plain, stacked] = scenesOf(clip).map((s) => must(groupOf(s), "scene").tracks);
    expect(plain).toHaveLength(1);
    expect(stacked).toHaveLength(2);
  });

  it("starts a scene's layers together, leaving their landing to the sequencer", () => {
    const clip = sceneSequence("s", [{ port: "in1", clip: layered([0, 12]) }], {
      gap: 0,
      sequencer,
    });
    expect(scenesOf(clip)[0].durationInFrames).toBe(30);
    const layers = must(groupOf(scenesOf(clip)[0]), "scene").tracks.map((t) => t.items[0]);
    expect(layers.map((l) => l.from)).toEqual([0, 0]);
    expect(layers.map((l) => l.handles?.move)).toEqual([undefined, undefined]);
    expect(layers.map((l) => l.slot)).toEqual([
      { nodeId: "l", port: "in1" },
      { nodeId: "l", port: "in2" },
    ]);
  });

  it("keeps a Layers length that follows one layer", () => {
    const short = { ...layered([0, 12]), durationInFrames: 20 };
    const clip = sceneSequence("s", [{ port: "in1", clip: short }], { gap: 0, sequencer });
    expect(clip.durationInFrames).toBe(20);
  });

  it("moves a layer's sound back with it and every scene's sound to the scene's start", () => {
    const front = { ...solid("y", 30), audio: [popSound("y")] };
    const clip = sceneSequence(
      "s",
      [
        { port: "in1", clip: { ...solid("p", 40), audio: [popSound("p")] } },
        { port: "in2", clip: layered([0, 12], front) },
      ],
      { gap: 2, sequencer },
    );
    expect(clip.audio.map((a) => a.from)).toEqual([0, 42]);
  });
});

describe("built-in items", () => {
  const log = {
    viewport: { width: 1280, height: 720 },
    dpr: 1,
    t0: 0,
    durationMs: 1000,
    actions: [],
  };
  const recording = () =>
    must(
      recordingClip(
        "rec",
        { ...defaultRecordingProps(), src: "project:take.mp4", name: "Take", log },
        30,
      ).tracks[0].items[0],
      "recording item",
    );

  it("reads a recording's and a group's props by their kind", () => {
    const item = recording();
    expect(item.kind).toBe("recording");
    expect(recordingOf(item)).toBe(item.props);
    const group = groupItem(
      { id: "g", from: 0, durationInFrames: 30 },
      { tracks: [{ id: "t", items: [item] }], remap: [[0, 0]], crop: { x: 0, y: 0, w: 1, h: 1 } },
    );
    expect(groupOf(group)).toBe(group.props);
    expect(groupOf(item)).toBeNull();
    expect(recordingOf(group)).toBeNull();
  });

  it("compiles every built-in item to its kind", () => {
    const placeholder = itemClip("p", "core/placeholder", { title: "T", subtitle: "S" }, 30);
    const kinds = sequenceClips("s", [placeholder, wrapClip(placeholder, "w")]).tracks.flatMap(
      (t) => t.items.map((i) => i.kind),
    );
    expect(kinds).toEqual(["group", "group"]);
    expect(placeholder.tracks[0].items[0]).toMatchObject({
      kind: "element",
      component: "core/placeholder",
    });
    expect(recording().kind).toBe("recording");
  });
});

describe("wrapSounds", () => {
  const beep = (from: number, durationInFrames?: number): AudioItem => ({
    id: "b",
    src: "project:beeps.wav",
    from,
    ...(durationInFrames === undefined ? {} : { durationInFrames }),
  });
  const placed = (items: AudioItem[]) =>
    items.map((a) => [a.from, a.durationInFrames, a.trimBefore ?? 0, a.rate ?? 1]);

  it("cuts sounds with a trimmed start, keeping their place in the picture", () => {
    expect(placed(wrapSounds([beep(0, 120)], { startFrom: 30 }, 90))).toEqual([[0, 90, 30, 1]]);
    expect(placed(wrapSounds([beep(60, 10)], { startFrom: 30 }, 90))).toEqual([[30, 10, 0, 1]]);
    expect(wrapSounds([beep(10, 10)], { startFrom: 30 }, 90)).toEqual([]);
  });

  it("repeats sounds with each repeat of the clip", () => {
    const out = wrapSounds([beep(15, 10)], { loopEvery: 60 }, 180);
    expect(out.map((a) => [a.id, a.from])).toEqual([
      ["b", 15],
      ["b:r1", 75],
      ["b:r2", 135],
    ]);
  });

  it("speeds sounds up with the clip", () => {
    expect(
      placed(
        wrapSounds(
          [beep(30, 60)],
          {
            remap: [
              [0, 0],
              [60, 120],
            ],
          },
          60,
        ),
      ),
    ).toEqual([[15, 30, 0, 2]]);
  });

  it("stops sounds where the clip ends", () => {
    expect(placed(wrapSounds([beep(0)], {}, 50))).toEqual([[0, 50, 0, 1]]);
  });
});

describe("sequence transitions", () => {
  const fade = (frames: number, hold = false): TransitionRef => ({
    kind: "fade",
    durationInFrames: frames,
    ...(hold ? { hold } : {}),
  });
  const spans = (clips: number[], cuts: Array<TransitionRef | undefined>) => {
    const seq = sequenceClips(
      "s",
      clips.map((f) => emptyClip(f)),
      cuts,
    );
    return {
      film: seq.durationInFrames,
      items: seq.tracks[0].items.map((i) => [i.from, i.from + i.durationInFrames]),
      cuts: seq.tracks[0].items.map((i) => i.enter?.durationInFrames ?? 0),
    };
  };
  const most = (items: number[][], film: number) =>
    Math.max(
      ...Array.from({ length: film }, (_, f) => items.filter(([a, b]) => f >= a && f < b).length),
    );

  it("never lets one clip's two transitions overlap, so at most two clips show", () => {
    const { film, items, cuts } = spans([30, 30, 30], [undefined, fade(60), fade(60)]);
    expect(cuts).toEqual([0, 15, 15]);
    expect(film).toBe(60);
    expect(most(items, film)).toBe(2);
  });

  it("lets a transition between two clips use nearly all of the shorter one", () => {
    expect(spans([30, 60], [undefined, fade(60)]).cuts).toEqual([0, 29]);
  });

  it("drops a transition into a one-frame clip", () => {
    expect(spans([30, 1], [undefined, fade(10)]).cuts).toEqual([0, 0]);
  });

  it("keeps a held transition at its length, since the clips don't overlap", () => {
    const { film, cuts } = spans([30, 30], [undefined, fade(45, true)]);
    expect(cuts).toEqual([0, 45]);
    expect(film).toBe(105);
  });
});
