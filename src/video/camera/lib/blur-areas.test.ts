import { describe, expect, it } from "vitest";
import type { BlurArea } from "../../../shared/schema/recording";
import { blurParams } from "./blur-areas";

const area = (extra: Partial<BlurArea> = {}): BlurArea => ({
  x: 0.1,
  y: 0.2,
  width: 0.3,
  height: 0.1,
  strength: 0.02,
  softness: 0.005,
  roundness: 0.4,
  ...extra,
});

describe("blur areas on a recording", () => {
  it("turns an area into its corners on the frame", () => {
    const [p] = blurParams([area()], 0, 1920);
    expect(p.topLeft).toEqual([0.1, 0.2]);
    expect(p.bottomRight[0]).toBeCloseTo(0.4);
    expect(p.bottomRight[1]).toBeCloseTo(0.3);
    expect(p.roundness).toBe(0.4);
  });

  it("keeps the corners on the frame", () => {
    const [p] = blurParams([area({ x: 0.9, width: 0.3, y: -0.1 })], 0, 1920);
    expect(p.topLeft).toEqual([0.9, 0]);
    expect(p.bottomRight[0]).toBe(1);
  });

  it("blurs the same share of the width in the small preview copy and the full take", () => {
    const [small] = blurParams([area()], 0, 960);
    const [full] = blurParams([area()], 0, 1920);
    expect(small.blurRadius / 960).toBeCloseTo(full.blurRadius / 1920);
    expect(small.feather / 960).toBeCloseTo(full.feather / 1920);
    expect(full.blurRadius).toBeCloseTo(38.4);
  });

  it("eases in after its start and out before its end", () => {
    const timed = [area({ fromMs: 1000, untilMs: 3000 })];
    const strength = (t: number) => blurParams(timed, t, 1000)[0].blurRadius;
    expect(strength(1000)).toBe(0);
    expect(strength(1100)).toBeCloseTo(10);
    expect(strength(2000)).toBeCloseTo(20);
    expect(strength(2900)).toBeCloseTo(10);
    expect(strength(3000)).toBe(0);
  });

  it("is off outside its times and on for the whole clip without them", () => {
    const timed = [area({ fromMs: 1000, untilMs: 3000 })];
    expect(blurParams(timed, 500, 1000)[0].disabled).toBe(true);
    expect(blurParams(timed, 3500, 1000)[0].disabled).toBe(true);
    expect(blurParams(timed, 2000, 1000)[0].disabled).toBe(false);
    expect(blurParams([area()], 0, 1000)[0]).toMatchObject({ disabled: false, blurRadius: 20 });
  });
});
