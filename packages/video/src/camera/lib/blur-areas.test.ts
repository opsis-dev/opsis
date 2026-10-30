import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { blurParams } from "#video/camera/lib/blur-areas.ts";
import type { BlurArea } from "#video/generated/spec.ts";

function unit(min: number, max: number) {
  return fc.double({ min, max, noNaN: true });
}

const area: fc.Arbitrary<BlurArea> = fc.record({
  x: unit(-0.5, 1.5),
  y: unit(-0.5, 1.5),
  width: unit(0, 1.5),
  height: unit(0, 1.5),
  strength: fc.integer({ min: 0, max: 100 }).map((n) => n / 1000),
  softness: unit(0, 0.05),
  roundness: unit(0, 1),
  fadeMs: fc.integer({ min: 1, max: 2000 }),
});

const timed = fc
  .tuple(area, fc.integer({ min: 0, max: 20_000 }), fc.integer({ min: 1, max: 20_000 }))
  .map(([a, fromMs, length]): BlurArea => ({ ...a, fromMs, untilMs: fromMs + length }));

describe("blur areas on a recording", () => {
  it("keeps every area's corners on the frame, in order", () => {
    fc.assert(
      fc.property(area, fc.integer({ min: 100, max: 4000 }), (a, width) => {
        const [p] = blurParams([a], 0, width);
        for (const v of [...p.topLeft, ...p.bottomRight]) {
          expect(v).toBeGreaterThanOrEqual(0);
          expect(v).toBeLessThanOrEqual(1);
        }
        expect(p.bottomRight[0]).toBeGreaterThanOrEqual(p.topLeft[0]);
        expect(p.bottomRight[1]).toBeGreaterThanOrEqual(p.topLeft[1]);
        expect(p.roundness).toBe(a.roundness);
      }),
    );
  });

  it("blurs the same share of the width whatever the width of the file", () => {
    fc.assert(
      fc.property(area, unit(0, 30_000), fc.integer({ min: 100, max: 4000 }), (a, t, width) => {
        const [small] = blurParams([a], t, width);
        const [full] = blurParams([a], t, width * 2);
        expect(full.blurRadius).toBeCloseTo(small.blurRadius * 2, 9);
        expect(full.feather).toBeCloseTo(small.feather * 2, 9);
      }),
    );
  });

  it("is off outside its times, on inside them and never stronger than set", () => {
    fc.assert(
      fc.property(timed, fc.integer({ min: -1000, max: 45_000 }), (a, t) => {
        const [p] = blurParams([a], t, 1000);
        const inside = t > (a.fromMs ?? 0) && t < (a.untilMs ?? 0);
        expect(p.disabled).toBe(!inside);
        expect(p.blurRadius).toBeGreaterThanOrEqual(0);
        expect(p.blurRadius).toBeLessThanOrEqual(a.strength * 1000 + 1e-9);
      }),
    );
  });

  it("eases in after its start and out before its end", () => {
    const timedArea = {
      x: 0.1,
      y: 0.2,
      width: 0.3,
      height: 0.1,
      strength: 0.02,
      softness: 0.005,
      roundness: 0.4,
      fadeMs: 200,
      fromMs: 1000,
      untilMs: 3000,
    };
    function strength(t: number) {
      return blurParams([timedArea], t, 1000)[0].blurRadius;
    }
    expect(strength(1100)).toBeCloseTo(10);
    expect(strength(2000)).toBeCloseTo(20);
    expect(strength(2900)).toBeCloseTo(10);
    fc.assert(
      fc.property(unit(1000, 2000), unit(0, 1000), (t, later) => {
        expect(strength(Math.min(2000, t + later))).toBeGreaterThanOrEqual(strength(t) - 1e-9);
        expect(strength(4000 - t)).toBeCloseTo(strength(t), 9);
      }),
    );
  });

  it("starts and stops at full strength without a fade", () => {
    fc.assert(
      fc.property(timed, fc.integer({ min: -1000, max: 45_000 }), (a, t) => {
        const sharp = { ...a, fadeMs: 0 };
        const [p] = blurParams([sharp], t, 1000);
        const inside = t >= (sharp.fromMs ?? 0) && t <= (sharp.untilMs ?? 0);
        expect(p.blurRadius).toBeCloseTo(inside ? sharp.strength * 1000 : 0, 9);
      }),
    );
  });

  it("stays on for the whole clip without times", () => {
    fc.assert(
      fc.property(area, unit(0, 1e6), (a, t) => {
        fc.pre(a.strength > 0);
        expect(blurParams([a], t, 1000)[0]).toMatchObject({
          disabled: false,
          blurRadius: a.strength * 1000,
        });
      }),
    );
  });
});
