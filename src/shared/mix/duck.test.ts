import { describe, expect, it } from "vitest";
import { groupItem } from "../compile/clip";
import type { AudioItem, TimelineItem, TimelineSpec } from "../schema/timeline";
import { duckGain, duckingSpans, type Span } from "./duck";

const effect = (id: string, from: number, extra: Partial<AudioItem> = {}): AudioItem => ({
  id,
  src: "app:sfx/whoosh.wav",
  from,
  role: "effect",
  lengthFrames: 10,
  channel: id,
  ...extra,
});

const keys = (...ids: string[]) => new Set(ids);

type Place = { from: number; durationInFrames: number };

const videoItem = (
  id: string,
  { muted = false, ...place }: Place & { muted?: boolean },
): TimelineItem => ({
  id,
  ...place,
  component: "core/video",
  props: { src: `project:${id}.mp4`, muted },
  nodeId: id,
});

const groupWith = (
  id: string,
  { loopEvery, ...place }: Place & { loopEvery?: number },
  items: TimelineItem[],
): TimelineItem =>
  groupItem({ id, ...place }, { tracks: [{ id: `${id}:inner`, items }], loopEvery });

const trackOf = (item: TimelineItem): TimelineSpec["tracks"] => [{ id: "t", items: [item] }];

const spec = (audio: AudioItem[], tracks: TimelineSpec["tracks"] = []) => ({
  fps: 30,
  durationInFrames: 300,
  audio,
  tracks,
});

describe("ducking", () => {
  it("merges overlapping spans", () => {
    const spans = duckingSpans(
      spec([effect("a", 20), effect("b", 0), effect("c", 5, { lengthFrames: 7 }), effect("d", 30)]),
      keys("a", "b", "c", "d"),
    );
    expect(spans).toEqual([
      [0, 12],
      [20, 40],
    ]);
  });

  it("lowers under the key channels but not under other or keep-music sounds", () => {
    const spans = duckingSpans(
      spec([
        { id: "m", src: "song.mp3", from: 0, role: "music", channel: "m" },
        effect("a", 30),
        effect("b", 100, { keepMusic: true }),
      ]),
      keys("a", "b", "v"),
    );
    expect(spans).toEqual([[30, 40]]);
  });

  it("plays effects of unknown length to the end, as the exported sound does, unless placed shorter", () => {
    expect(duckingSpans(spec([effect("a", 0, { lengthFrames: undefined })]), keys("a"))).toEqual([
      [0, 300],
    ]);
    expect(
      duckingSpans(
        spec([effect("b", 100, { lengthFrames: undefined, durationInFrames: 12 })]),
        keys("b"),
      ),
    ).toEqual([[100, 112]]);
  });

  it("finds clips with sound inside groups", () => {
    const tracks = trackOf(
      groupWith("g", { from: 50, durationInFrames: 100 }, [
        videoItem("v", { from: 10, durationInFrames: 200 }),
        videoItem("q", { from: 0, durationInFrames: 20, muted: true }),
      ]),
    );
    expect(duckingSpans(spec([effect("a", 0)], tracks), keys("v", "q"))).toEqual([[60, 150]]);
    expect(duckingSpans(spec([effect("a", 0)], tracks), keys("a", "v", "q"))).toEqual([
      [0, 10],
      [60, 150],
    ]);
  });

  it("lowers under every repeat of a looped clip with sound", () => {
    const tracks = trackOf(
      groupWith("g", { from: 20, durationInFrames: 90, loopEvery: 30 }, [
        videoItem("v", { from: 0, durationInFrames: 30 }),
      ]),
    );
    expect(duckingSpans(spec([], tracks), keys("v"))).toEqual([[20, 110]]);
  });

  const spans: Span[] = [[30, 40]];

  it.each([
    [0, 10, 1],
    [20, 10, 1],
    [25, 10, 0.65],
    [30, 10, 0.3],
    [39, 10, 0.3],
    [45, 10, 0.65],
    [50, 10, 1],
    [35, 0, 0.3],
    [29, 0, 1],
  ])("ramps around a sound: frame %i with a %i-frame fade is at %f", (frame, fade, gain) => {
    expect(duckGain(frame, spans, 0.3, fade)).toBeCloseTo(gain);
  });
});
