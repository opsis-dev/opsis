import { describe, expect, it } from "vitest";
import { inverseRemap, remapDuration, remapFrame, remapFromSegments } from "#core/compile/remap";
import type { Remap } from "#core/schema/timeline";

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

describe("inverseRemap", () => {
  const fast: Remap = [
    [0, 0],
    [10, 40],
    [20, 50],
  ];

  it("finds the output frame of an inner frame on a sped-up part", () => {
    expect(inverseRemap(fast, 20, "start")).toBe(5);
    expect(inverseRemap(fast, 20, "end")).toBe(5);
  });

  it("carries on at normal speed past the last point", () => {
    expect(inverseRemap(fast, 55, "start")).toBe(25);
    expect(inverseRemap(fast, 55, "end")).toBe(25);
  });

  it("gives the first and last output frame of a held frame", () => {
    const held: Remap = [
      [0, 0],
      [10, 10],
      [20, 10],
      [30, 20],
    ];
    expect(inverseRemap(held, 10, "start")).toBe(10);
    expect(inverseRemap(held, 10, "end")).toBe(20);
  });

  it("extends the first part's speed before the first point", () => {
    const offset: Remap = [
      [5, 10],
      [15, 20],
    ];
    expect(inverseRemap(offset, 4, "start")).toBe(-1);
  });
});
