import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import type { TerminalEffects } from "#video/generated/spec.ts";
import { hasLens, softenPx, trailWeights } from "#video/terminal/effects.tsx";

const NONE: TerminalEffects = {
  soften: 0,
  vignette: 0,
  vignetteX: 0.5,
  vignetteY: 0.5,
  vignetteFollowsFrame: false,
  grain: 0,
  colourGrain: true,
  fringe: 0,
  fringeAngle: 0,
  bulge: 0,
  bulgeX: 0.5,
  bulgeY: 0.5,
  trails: 0,
};

describe("terminal page effects", () => {
  it("fades each earlier frame of a trail more than the one after it", () => {
    fc.assert(
      fc.property(fc.double({ min: 0.01, max: 1, noNaN: true }), (trails) => {
        const weights = trailWeights({ ...NONE, trails });
        expect(weights.length).toBeGreaterThan(0);
        expect(weights.every((w) => w > 0 && w < 1)).toBe(true);
        expect(weights.slice(1).every((w, i) => w < (weights[i] ?? 0))).toBe(true);
      }),
    );
    expect(trailWeights({ ...NONE, trails: 0 })).toEqual([]);
    expect(trailWeights(undefined)).toEqual([]);
  });

  it("softens in proportion to the screen's width", () => {
    fc.assert(
      fc.property(
        fc.double({ min: 0, max: 1, noNaN: true }),
        fc.integer({ min: 100, max: 4000 }),
        (soften, width) => {
          const one = softenPx({ ...NONE, soften }, { width, height: 100 });
          const two = softenPx({ ...NONE, soften }, { width: width * 2, height: 100 });
          expect(two).toBeCloseTo(one * 2, 9);
        },
      ),
    );
  });

  it("filters the screen only for fringing or a bulge", () => {
    expect(hasLens(NONE)).toBe(false);
    expect(hasLens({ ...NONE, fringe: 0.2 })).toBe(true);
    expect(hasLens({ ...NONE, bulge: -0.3 })).toBe(true);
    expect(hasLens(undefined)).toBe(false);
  });
});
