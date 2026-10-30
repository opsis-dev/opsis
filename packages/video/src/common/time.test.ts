import { describe, expect, it } from "vitest";
import { playRange } from "#video/common/time.ts";

describe("the range the playhead plays between in and out points", () => {
  it("plays everything when neither point is set or the film is a frame long", () => {
    expect(playRange(null, null, 300)).toEqual({});
    expect(playRange(10, 20, 1)).toEqual({});
  });

  it("fills a missing point with the film's edge", () => {
    expect(playRange(30, null, 300)).toEqual({ inFrame: 30, outFrame: 299 });
    expect(playRange(null, 120, 300)).toEqual({ inFrame: 0, outFrame: 120 });
  });

  it("keeps the out point after the in point and both inside the film, even with the in point on the last frame", () => {
    expect(playRange(299, null, 300)).toEqual({ inFrame: 298, outFrame: 299 });
    expect(playRange(100, 50, 300)).toEqual({ inFrame: 100, outFrame: 101 });
    expect(playRange(-5, 900, 300)).toEqual({ inFrame: 0, outFrame: 299 });
  });
});
