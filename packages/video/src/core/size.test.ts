import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { containBox } from "#video/core/size.ts";

function size(min: number, max: number) {
  return fc.record({ width: fc.integer({ min, max }), height: fc.integer({ min, max }) });
}

function aspect(s: { width: number; height: number }) {
  return s.width / s.height;
}

describe("containBox", () => {
  it("fits the shape inside the frame, touching two of its sides", () => {
    fc.assert(
      fc.property(size(1, 5000), fc.double({ min: 0.1, max: 10, noNaN: true }), (outer, ratio) => {
        const box = containBox(outer, ratio);
        expect(box.width).toBeLessThanOrEqual(outer.width + 1e-6);
        expect(box.height).toBeLessThanOrEqual(outer.height + 1e-6);
        expect(aspect(box)).toBeCloseTo(ratio, 6);
        expect(
          Math.abs(box.width - outer.width) < 1e-6 || Math.abs(box.height - outer.height) < 1e-6,
        ).toBe(true);
      }),
    );
  });
});
