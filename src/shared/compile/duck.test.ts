import { describe, expect, it } from "vitest";
import type { AudioItem, TimelineSpec } from "../schema/timeline";
import { duckGain, duckingSpans, mergeSpans, type Span } from "./duck";

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

const spec = (audio: AudioItem[], tracks: TimelineSpec["tracks"] = []) => ({
  fps: 30,
  durationInFrames: 300,
  audio,
  tracks,
});

describe("ducking", () => {
  it("merges overlapping spans", () => {
    expect(
      mergeSpans([
        [20, 30],
        [0, 10],
        [5, 12],
        [30, 40],
      ]),
    ).toEqual([
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

  it("uses one second for effects of unknown length, capped by their placed length", () => {
    const spans = duckingSpans(
      spec([
        effect("a", 0, { lengthFrames: undefined }),
        effect("b", 100, { lengthFrames: undefined, durationInFrames: 12 }),
      ]),
      keys("a", "b"),
    );
    expect(spans).toEqual([
      [0, 30],
      [100, 112],
    ]);
  });

  it("finds clips with sound inside groups", () => {
    const tracks: TimelineSpec["tracks"] = [
      {
        id: "t",
        items: [
          {
            id: "g",
            from: 50,
            durationInFrames: 100,
            component: "core/group",
            props: {
              tracks: [
                {
                  id: "inner",
                  items: [
                    {
                      id: "v",
                      from: 10,
                      durationInFrames: 200,
                      component: "core/video",
                      props: { muted: false },
                      nodeId: "v",
                    },
                    {
                      id: "quiet",
                      from: 0,
                      durationInFrames: 20,
                      component: "core/video",
                      props: { muted: true },
                      nodeId: "q",
                    },
                  ],
                },
              ],
            },
          },
        ],
      },
    ];
    expect(duckingSpans(spec([effect("a", 0)], tracks), keys("v", "q"))).toEqual([[60, 150]]);
    expect(duckingSpans(spec([effect("a", 0)], tracks), keys("a", "v", "q"))).toEqual([
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
