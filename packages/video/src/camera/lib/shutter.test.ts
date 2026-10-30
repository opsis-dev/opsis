import * as fc from "fast-check";
import { expect, it } from "vitest";
import { sampleOpacity, shutterOffsets } from "#video/camera/lib/shutter.ts";

const angle = fc.double({ min: -90, max: 450, noNaN: true });
const samples = fc.integer({ min: 0, max: 32 });

it("starts at the current frame and stays inside the open shutter, earliest last", () => {
  fc.assert(
    fc.property(angle, samples, (shutterAngle, count) => {
      const offsets = shutterOffsets(shutterAngle, count);
      const open = Math.min(Math.max(shutterAngle, 0), 360) / 360;
      expect(offsets[0]).toBe(0);
      expect(offsets.length).toBe(open === 0 || count <= 1 ? 1 : count);
      expect(offsets.every((offset) => offset < Math.max(open, Number.EPSILON))).toBe(true);
      expect(offsets.slice(1).every((offset, i) => offset > (offsets[i] ?? 0))).toBe(true);
    }),
  );
});

function shareOf(layer: number, count: number): number {
  let share = sampleOpacity(layer);
  for (let above = layer + 1; above < count; above++) share *= 1 - sampleOpacity(above);
  return share;
}

it("gives every stacked sample the same share of the picture", () => {
  fc.assert(
    fc.property(fc.integer({ min: 1, max: 64 }), (count) => {
      const shares = Array.from({ length: count }, (_, layer) => shareOf(layer, count));
      expect(shares.every((share) => Math.abs(share - 1 / count) < 1e-9)).toBe(true);
    }),
  );
});
