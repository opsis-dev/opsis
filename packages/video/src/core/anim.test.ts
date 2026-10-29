import * as fc from "fast-check";
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

  it("stays between nothing and whole and is whole between its ramps", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 300 }),
        fc.integer({ min: 0, max: 60 }),
        fc.integer({ min: 0, max: 60 }),
        fc.double({ min: 0, max: 1, noNaN: true }),
        (length, fadeIn, fadeOut, share) => {
          const end = length + fadeIn + fadeOut;
          const value = fadeWindow(share * end, 0, end, { in: fadeIn, out: fadeOut });
          expect(value).toBeGreaterThanOrEqual(0);
          expect(value).toBeLessThanOrEqual(1);
          const between = share * end >= fadeIn && share * end <= fadeIn + length;
          expect(between ? Math.abs(value - 1) : 0).toBeLessThan(1e-9);
        },
      ),
    );
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

  it("stays between nothing and whole and never undraws as time goes on", () => {
    fc.assert(
      fc.property(
        fc.double({ min: -10, max: 10, noNaN: true }),
        fc.double({ min: -10, max: 10, noNaN: true }),
        fc.double({ min: 0, max: 10, noNaN: true }),
        fc.double({ min: 0.01, max: 5, noNaN: true }),
        (t, from, later, length) => {
          const now = drawOn(t, from, length);
          expect(now).toBeGreaterThanOrEqual(0);
          expect(now).toBeLessThanOrEqual(1);
          expect(drawOn(t + later, from, length)).toBeGreaterThanOrEqual(now);
          expect(drawOn(from - later - 0.001, from, length)).toBe(0);
          expect(drawOn(from + length + later, from, length)).toBe(1);
        },
      ),
    );
  });
});
