import { describe, expect, it } from "vitest";
import { framesToMs, msToFrames } from "./time";

describe("ms and frames", () => {
  it("rounds to whole frames with a minimum", () => {
    expect(msToFrames(1000, 30)).toBe(30);
    expect(msToFrames(0, 30)).toBe(1);
    expect(msToFrames(0, 30, 0)).toBe(0);
  });

  it("rounds frames to whole milliseconds", () => {
    expect(framesToMs(30, 30)).toBe(1000);
    expect(framesToMs(1, 30)).toBe(33);
  });
});
