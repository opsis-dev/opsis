import * as fc from "fast-check";
import { expect, it } from "vitest";
import { clockPoints } from "#video/elements/transition/clock-wipe.tsx";
import { flipAngle } from "#video/elements/transition/flip.tsx";
import { irisRadius } from "#video/elements/transition/iris.tsx";
import { slideOffset } from "#video/elements/transition/slide.tsx";
import { wipeCorners } from "#video/elements/transition/wipe.tsx";
import type { Direction, WipeDirection } from "#video/generated/spec.ts";

const DIRECTIONS: Direction[] = ["from-left", "from-right", "from-top", "from-bottom"];
const WIPES: WipeDirection[] = [
  ...DIRECTIONS,
  "from-top-left",
  "from-top-right",
  "from-bottom-left",
  "from-bottom-right",
];
const progress = fc.double({ min: 0, max: 1, noNaN: true });
const unit = fc.double({ min: 0.001, max: 0.999, noNaN: true });
const frame = fc.record({
  width: fc.integer({ min: 16, max: 4096 }),
  height: fc.integer({ min: 16, max: 4096 }),
});

function inside([x, y]: [number, number], polygon: Array<[number, number]>): boolean {
  let hit = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const [xi, yi] = polygon[i];
    const [xj, yj] = polygon[j];
    if (yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) hit = !hit;
  }
  return hit;
}

it("wipes nothing in at the start, everything at the end, and never hides what it showed", () => {
  fc.assert(
    fc.property(
      fc.constantFrom(...WIPES),
      progress,
      progress,
      unit,
      unit,
      (direction, a, b, x, y) => {
        const [early, late] = a < b ? [a, b] : [b, a];
        expect(inside([x, y], wipeCorners(direction, 0))).toBe(false);
        expect(inside([x, y], wipeCorners(direction, 1))).toBe(true);
        const shownEarly = inside([x, y], wipeCorners(direction, early));
        expect(!shownEarly || inside([x, y], wipeCorners(direction, late))).toBe(true);
      },
    ),
  );
});

it("sweeps the clock from nothing to the whole frame without hiding what it showed", () => {
  fc.assert(
    fc.property(frame, progress, progress, unit, unit, (size, a, b, x, y) => {
      const [early, late] = a < b ? [a, b] : [b, a];
      const spot: [number, number] = [x * size.width, y * size.height];
      expect(inside(spot, clockPoints(size, 0))).toBe(false);
      fc.pre(Math.abs(spot[0] - size.width / 2) > 1e-6 || spot[1] > size.height / 2);
      expect(inside(spot, clockPoints(size, 1))).toBe(true);
      const shownEarly = inside(spot, clockPoints(size, early));
      expect(!shownEarly || inside(spot, clockPoints(size, late))).toBe(true);
    }),
  );
});

it("opens the iris until it reaches the corners", () => {
  fc.assert(
    fc.property(frame, (size) => {
      expect(irisRadius(size, 0)).toBe(0);
      expect(irisRadius(size, 1)).toBeCloseTo(Math.hypot(size.width / 2, size.height / 2), 6);
    }),
  );
});

it("slides both scenes together, the new one ending in place", () => {
  fc.assert(
    fc.property(fc.constantFrom(...DIRECTIONS), progress, (direction, p) => {
      const gap = slideOffset(direction, true, p) - slideOffset(direction, false, p);
      expect(Math.abs(gap)).toBeCloseTo(1, 9);
      expect(slideOffset(direction, true, 1)).toBeCloseTo(0, 9);
      expect(slideOffset(direction, false, 0)).toBeCloseTo(0, 9);
    }),
  );
});

it("turns the new scene as the back of the old one, ending face on", () => {
  fc.assert(
    fc.property(fc.constantFrom(...DIRECTIONS), progress, (direction, p) => {
      const between = flipAngle(direction, true, p) - flipAngle(direction, false, p);
      expect(Math.abs(between)).toBeCloseTo(180, 9);
      expect(flipAngle(direction, true, 1)).toBeCloseTo(0, 9);
      expect(flipAngle(direction, false, 0)).toBeCloseTo(0, 9);
    }),
  );
});
