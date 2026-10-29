import { describe, expect, it } from "vitest";
import { duckGain, type Span } from "./duck";

describe("ducking", () => {
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
