import { describe, expect, it } from "vitest";
import { drawOn, fadeWindow } from "#video/core/anim.ts";

describe("fadeWindow", () => {
  it("ramps in and out over the window", () => {
    const fade = { in: 2, out: 4 };
    expect(fadeWindow(0, 0, 10, fade)).toBe(0);
    expect(fadeWindow(1, 0, 10, fade)).toBe(0.5);
    expect(fadeWindow(5, 0, 10, fade)).toBe(1);
    expect(fadeWindow(8, 0, 10, fade)).toBe(0.5);
    expect(fadeWindow(10, 0, 10, fade)).toBe(0);
  });

  it("skips a ramp of zero length", () => {
    expect(fadeWindow(0, 0, 10, { in: 0, out: 5 })).toBe(1);
    expect(fadeWindow(10, 0, 10, { in: 5, out: 0 })).toBe(1);
  });
});

describe("drawOn", () => {
  it("draws a mark in from nothing to whole over its length", () => {
    expect(drawOn(1, 2)).toBe(0);
    expect(drawOn(2, 2)).toBe(0);
    expect(drawOn(2.225, 2)).toBeCloseTo(0.5);
    expect(drawOn(2.45, 2)).toBe(1);
    expect(drawOn(9, 2)).toBe(1);
  });

  it("gives the same value at a time whichever way the playhead moves", () => {
    const forward = [0, 0.1, 0.2, 0.3, 0.4].map((t) => drawOn(t, 0, 0.4));
    const backward = [0.4, 0.3, 0.2, 0.1, 0].map((t) => drawOn(t, 0, 0.4)).toReversed();
    expect(backward).toEqual(forward);
    expect(forward).toEqual([...forward].sort((a, b) => a - b));
  });
});
