import { describe, expect, it } from "vitest";
import { remapDuration, remapFrame, remapFromSegments, type Remap } from "./remap";

describe("remaps", () => {
  it("maps output frames to inner frames piecewise-linearly", () => {
    const remap: Remap = [
      [0, 0],
      [10, 40],
      [20, 50],
    ];
    expect(remapFrame(remap, 5)).toBe(20);
    expect(remapFrame(remap, 15)).toBe(45);
    expect(remapFrame(remap, 25)).toBe(55);
  });

  it("builds a remap from speed segments", () => {
    const remap = remapFromSegments(100, [{ from: 20, to: 60, rate: 4 }]);
    expect(remapDuration(remap)).toBe(70);
    expect(remapFrame(remap, 30)).toBe(60);
  });
});
