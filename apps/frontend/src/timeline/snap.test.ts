import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { rulerStep, snapFrame, snapSpan } from "./snap";

const target = fc.record({
  frame: fc.integer({ min: 0, max: 2000 }),
  kind: fc.constantFrom("playhead" as const, "edge" as const, "marker" as const, "beat" as const),
});

describe("snapping", () => {
  it("snaps to the nearest target inside the threshold and stays put otherwise", () => {
    fc.assert(
      fc.property(
        fc.array(target, { maxLength: 12 }),
        fc.double({ min: -100, max: 2100, noNaN: true }),
        fc.integer({ min: 0, max: 40 }),
        (targets, frame, threshold) => {
          const snapped = snapFrame(frame, targets, threshold);
          const nearest = Math.min(...targets.map((t) => Math.abs(t.frame - frame)));
          const inReach = nearest <= threshold;
          expect(snapped.target !== null).toBe(inReach);
          expect(snapped.frame).toBe(snapped.target ? snapped.target.frame : frame);
          expect(inReach ? Math.abs(snapped.frame - frame) : nearest).toBe(nearest);
        },
      ),
    );
  });

  it("puts one end of a moved span exactly on a target, whichever is closer", () => {
    fc.assert(
      fc.property(
        fc.array(target, { maxLength: 12 }),
        fc.integer({ min: 0, max: 1000 }),
        fc.integer({ min: 1, max: 500 }),
        fc.double({ min: -300, max: 300, noNaN: true }),
        fc.integer({ min: 0, max: 40 }),
        (targets, from, length, delta, threshold) => {
          const to = from + length;
          const snapped = snapSpan(from, to, delta, targets, threshold);
          const landed = [from + snapped.delta, to + snapped.delta];
          const ends = [from, to].map((edge) => snapFrame(edge + delta, targets, threshold));
          const closest = Math.min(
            ...ends.map((own, i) =>
              own.target ? Math.abs(own.frame - ([from, to][i] + delta)) : Number.POSITIVE_INFINITY,
            ),
          );
          expect(snapped.target !== null).toBe(ends.some((own) => own.target));
          expect(
            snapped.target ? landed.includes(snapped.target.frame) : snapped.delta === delta,
          ).toBe(true);
          expect(
            snapped.target ? Math.abs(Math.abs(snapped.delta - delta) - closest) : 0,
          ).toBeLessThanOrEqual(1e-9);
        },
      ),
    );
  });
});

describe("ruler", () => {
  it("spaces ruler ticks by zoom", () => {
    expect(rulerStep(30, 30)).toEqual({ step: 5, minor: 1 });
    expect(rulerStep(1, 30)).toEqual({ step: 150, minor: 30 });
    expect(rulerStep(0.05, 30)).toEqual({ step: 1800, minor: 360 });
    expect(rulerStep(0.001, 30)).toEqual({ step: 18_000, minor: 1800 });
  });
});
