import { describe, expect, it } from "vitest";
import type { AudioBar, AudioItem, FlatTimeline } from "@/generated/bindings";
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

const clipBar = (channel: string, from: number, to: number): AudioBar => ({
  type: "audio",
  key: `${channel}:${from}`,
  id: channel,
  channel,
  src: `project:${channel}.mp4`,
  role: "clip",
  from,
  to,
  lane: 0,
  trimBefore: 0,
  loop: false,
  volume: 1,
});

const flat = (audio: AudioBar[] = []): Pick<FlatTimeline, "audio"> => ({ audio });

const spec = (audio: AudioItem[]) => ({ durationInFrames: 300, audio });

describe("ducking", () => {
  it("merges overlapping spans", () => {
    const spans = duckingSpans(
      spec([effect("a", 20), effect("b", 0), effect("c", 5, { lengthFrames: 7 }), effect("d", 30)]),
      keys("a", "b", "c", "d"),
      flat(),
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
      flat(),
    );
    expect(spans).toEqual([[30, 40]]);
  });

  it("plays effects of unknown length to the end, as the exported sound does, unless placed shorter", () => {
    expect(
      duckingSpans(spec([effect("a", 0, { lengthFrames: undefined })]), keys("a"), flat()),
    ).toEqual([[0, 300]]);
    expect(
      duckingSpans(
        spec([effect("b", 100, { lengthFrames: undefined, durationInFrames: 12 })]),
        keys("b"),
        flat(),
      ),
    ).toEqual([[100, 112]]);
  });

  it("lowers under the clips with sound on a key channel", () => {
    const bars = flat([clipBar("v", 60, 150), clipBar("q", 0, 20)]);
    expect(duckingSpans(spec([effect("a", 0)]), keys("v"), bars)).toEqual([[60, 150]]);
    expect(duckingSpans(spec([effect("a", 0)]), keys("a", "v"), bars)).toEqual([
      [0, 10],
      [60, 150],
    ]);
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
