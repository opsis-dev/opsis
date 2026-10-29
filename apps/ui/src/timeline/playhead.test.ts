import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { activeAt, loopOf, wrapInLoop } from "./playhead";

const span = fc
  .tuple(fc.integer({ min: 0, max: 1000 }), fc.integer({ min: 1, max: 500 }))
  .map(([from, length]) => ({ from, to: from + length }));

describe("activeAt", () => {
  it("lists exactly the nodes whose span holds the frame, sorted, ending before the last frame", () => {
    fc.assert(
      fc.property(
        fc.dictionary(fc.string({ minLength: 1, maxLength: 4 }), span),
        fc.integer({ min: 0, max: 1600 }),
        (spans, frame) => {
          const active = activeAt(spans, frame);
          expect(active).toEqual(active.toSorted());
          const holding = Object.entries(spans)
            .filter(([, s]) => s.from <= frame && frame < s.to)
            .map(([id]) => id);
          expect(active.toSorted()).toEqual(holding.toSorted());
        },
      ),
    );
  });
});

describe("loopOf", () => {
  it("loops the whole film without in and out points", () => {
    expect(loopOf(null, null, 300)).toEqual({ from: 0, to: 300 });
  });

  it("plays the out point's frame and stays inside the film for any points", () => {
    fc.assert(
      fc.property(
        fc.option(fc.integer({ min: -50, max: 400 })),
        fc.option(fc.integer({ min: -50, max: 400 })),
        fc.integer({ min: 1, max: 300 }),
        (inPoint, outPoint, duration) => {
          const loop = loopOf(inPoint, outPoint, duration);
          expect(loop.from).toBeGreaterThanOrEqual(0);
          expect(loop.to).toBeLessThanOrEqual(duration);
          expect(loop.from).toBeLessThan(loop.to);
        },
      ),
    );
    expect(loopOf(30, 120, 300)).toEqual({ from: 30, to: 121 });
  });
});

describe("wrapInLoop", () => {
  it("keeps every position inside the loop at the same place in its cycle", () => {
    fc.assert(
      fc.property(span, fc.double({ min: -10_000, max: 10_000, noNaN: true }), (loop, position) => {
        const wrapped = wrapInLoop(position, loop);
        expect(wrapped).toBeGreaterThanOrEqual(loop.from);
        expect(wrapped).toBeLessThan(loop.to);
        const cycles = (position - wrapped) / (loop.to - loop.from);
        expect(Math.abs(cycles - Math.round(cycles))).toBeLessThan(1e-6);
        const inside = position >= loop.from && position < loop.to;
        expect(inside ? wrapped : position).toBe(position);
      }),
    );
  });
});
