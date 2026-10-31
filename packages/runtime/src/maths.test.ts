import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { Easing } from "#runtime/easing.ts";
import { interpolate } from "#runtime/interpolate.ts";
import { random } from "#runtime/random.ts";
import { measureSpring, spring, springConfig, springProgress } from "#runtime/spring.ts";

const unit = fc.double({ min: 0, max: 1, noNaN: true });
const value = fc.double({ min: -1000, max: 1000, noNaN: true });

const rising = fc
  .array(fc.double({ min: 0.01, max: 100, noNaN: true }), { minLength: 1, maxLength: 5 })
  .chain((gaps) =>
    fc.tuple(
      fc.double({ min: -500, max: 500, noNaN: true }),
      fc.constant(gaps),
      fc.array(value, { minLength: gaps.length + 1, maxLength: gaps.length + 1 }),
    ),
  )
  .map(([start, gaps, outputs]) => {
    const inputs = gaps.reduce<number[]>((all, gap) => [...all, (all.at(-1) ?? 0) + gap], [start]);
    return { inputs, outputs };
  });

describe("interpolate", () => {
  it("passes through every stop and stays between neighbouring outputs", () => {
    fc.assert(
      fc.property(rising, unit, ({ inputs, outputs }, t) => {
        inputs.forEach((x, i) =>
          expect(interpolate(x, inputs, outputs)).toBeCloseTo(outputs[i] ?? 0, 6),
        );
        const i = Math.min(inputs.length - 2, Math.floor(t * (inputs.length - 1)));
        const x = (inputs[i] ?? 0) + ((inputs[i + 1] ?? 0) - (inputs[i] ?? 0)) * t;
        const out = interpolate(x, inputs, outputs);
        const [lo, hi] = [outputs[i] ?? 0, outputs[i + 1] ?? 0].toSorted((a, b) => a - b);
        expect(out).toBeGreaterThanOrEqual((lo ?? 0) - 1e-6);
        expect(out).toBeLessThanOrEqual((hi ?? 0) + 1e-6);
      }),
    );
  });

  it("clamps, extends, passes through or wraps outside the range as asked", () => {
    fc.assert(
      fc.property(
        rising,
        fc.double({ min: 0.01, max: 500, noNaN: true }),
        ({ inputs, outputs }, beyond) => {
          const first = inputs[0] ?? 0;
          const last = inputs.at(-1) ?? 0;
          const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
          expect(interpolate(first - beyond, inputs, outputs, clamp)).toBe(outputs[0]);
          expect(interpolate(last + beyond, inputs, outputs, clamp)).toBe(outputs.at(-1));
          const identity = { extrapolateLeft: "identity", extrapolateRight: "identity" } as const;
          expect(interpolate(last + beyond, inputs, outputs, identity)).toBe(last + beyond);
          const wrap = { extrapolateLeft: "wrap", extrapolateRight: "wrap" } as const;
          const span = last - first;
          expect(interpolate(last + span / 3, inputs, outputs, wrap)).toBeCloseTo(
            interpolate(first + span / 3, inputs, outputs),
            6,
          );
          const [a, b] = [inputs[0] ?? 0, inputs[1] ?? 0];
          const [c, d] = [outputs[0] ?? 0, outputs[1] ?? 0];
          expect(interpolate(a - beyond, inputs, outputs)).toBeCloseTo(
            c - ((d - c) / (b - a)) * beyond,
            4,
          );
        },
      ),
    );
  });

  it("keeps the ends and stays between them on a perceptual scale", () => {
    const size = fc.double({ min: 0.01, max: 50, noNaN: true });
    fc.assert(
      fc.property(size, size, unit, (from, to, t) => {
        const scale = { output: "perceptual-scale" } as const;
        expect(interpolate(0, [0, 1], [from, to], scale)).toBeCloseTo(from, 9);
        expect(interpolate(1, [0, 1], [from, to], scale)).toBeCloseTo(to, 9);
        const out = interpolate(t, [0, 1], [from, to], scale);
        expect(out).toBeGreaterThanOrEqual(Math.min(from, to) - 1e-9);
        expect(out).toBeLessThanOrEqual(Math.max(from, to) + 1e-9);
        expect(out ** 2).toBeCloseTo(from ** 2 + (to ** 2 - from ** 2) * t, 6);
      }),
    );
  });

  it("refuses ranges that don't rise or don't match", () => {
    expect(() => interpolate(0, [0, 0], [0, 1])).toThrow("rise strictly");
    expect(() => interpolate(0, [0, 1], [0])).toThrow("same length");
  });
});

describe("Easing", () => {
  const curves = [
    Easing.linear,
    Easing.ease,
    Easing.quad,
    Easing.cubic,
    Easing.poly(4),
    Easing.sin,
    Easing.circle,
    Easing.bounce,
    Easing.elastic(1),
    Easing.back(),
    Easing.bezier(0.16, 1, 0.3, 1),
    Easing.out(Easing.cubic),
    Easing.inOut(Easing.quad),
    Easing.spring({ damping: 18, stiffness: 120, mass: 1 }),
  ];

  it("starts at 0 and ends at 1", () => {
    for (const curve of curves) {
      expect(curve(0)).toBeCloseTo(0, 6);
      expect(curve(1)).toBeCloseTo(1, 6);
    }
    expect(Easing.exp(1)).toBe(1);
  });

  it("mirrors a curve for out and halves it for in and out", () => {
    fc.assert(
      fc.property(unit, (t) => {
        expect(Easing.out(Easing.cubic)(t)).toBeCloseTo(1 - (1 - t) ** 3, 9);
        const inOut = Easing.inOut(Easing.cubic);
        expect(inOut(t) + inOut(1 - t)).toBeCloseTo(1, 9);
      }),
    );
  });

  it("solves a bezier curve so its x matches the time asked", () => {
    fc.assert(
      fc.property(unit, unit, unit, unit, unit, (x1, y1, x2, y2, t) => {
        const curve = Easing.bezier(x1, y1, x2, y2);
        const straight = Easing.bezier(x1, x1, x2, x2);
        expect(straight(t)).toBeCloseTo(t, 6);
        expect(Number.isFinite(curve(t))).toBe(true);
      }),
    );
    fc.assert(
      fc.property(unit, unit, (a, b) => {
        const curve = Easing.bezier(0.42, 0, 0.58, 1);
        const [early, late] = a < b ? [a, b] : [b, a];
        expect(curve(early)).toBeLessThanOrEqual(curve(late) + 1e-9);
      }),
    );
  });
});

const config = fc.record({
  mass: fc.double({ min: 0.2, max: 5, noNaN: true }),
  damping: fc.double({ min: 2, max: 60, noNaN: true }),
  stiffness: fc.double({ min: 20, max: 400, noNaN: true }),
  overshootClamping: fc.boolean(),
});

describe("spring", () => {
  it("rests at 0 before it starts and settles on 1 by its measured length", () => {
    fc.assert(
      fc.property(config, fc.integer({ min: 0, max: 30 }), (c, delay) => {
        const fps = 30;
        expect(spring({ frame: delay, fps, config: c, delay })).toBe(0);
        const settled = measureSpring({ fps, config: c });
        for (let frame = settled; frame < settled + 60; frame += 7) {
          expect(Math.abs(1 - spring({ frame, fps, config: c }))).toBeLessThan(0.005);
        }
      }),
    );
  });

  it("moves from and to the values asked for", () => {
    fc.assert(
      fc.property(config, value, value, fc.integer({ min: 0, max: 120 }), (c, from, to, frame) => {
        const fps = 30;
        const unitSpring = spring({ frame, fps, config: c });
        expect(spring({ frame, fps, config: c, from, to })).toBeCloseTo(
          from + (to - from) * unitSpring,
          6,
        );
        expect(spring({ frame: 0, fps, config: c, from, to })).toBe(from);
      }),
    );
  });

  it("reaches its rest at the length asked for and stays on its target after it", () => {
    fc.assert(
      fc.property(
        config,
        fc.integer({ min: 5, max: 200 }),
        fc.integer({ min: 1, max: 100 }),
        value,
        (c, length, after, to) => {
          const end = spring({ frame: length, fps: 30, config: c, durationInFrames: length });
          expect(Math.abs(1 - end)).toBeLessThan(0.006);
          const options = { fps: 30, config: c, durationInFrames: length, to };
          expect(spring({ ...options, frame: length + after })).toBe(to);
        },
      ),
    );
  });

  it("never passes its target when overshoot is clamped", () => {
    fc.assert(
      fc.property(config, fc.double({ min: 0, max: 10, noNaN: true }), (c, seconds) => {
        expect(
          springProgress(springConfig({ ...c, overshootClamping: true }), seconds),
        ).toBeLessThanOrEqual(1 + 1e-12);
      }),
    );
  });

  it("moves like a damped spring, damping above critical moving critically damped: acceleration is minus stiffness times distance minus damping times speed", () => {
    fc.assert(
      fc.property(config, fc.double({ min: 0.01, max: 3, noNaN: true }), (c, seconds) => {
        const resolved = springConfig({ ...c, overshootClamping: false });
        const h = 1e-4;
        function at(s: number) {
          return 1 - springProgress(resolved, s);
        }
        const speed = (at(seconds + h) - at(seconds - h)) / (2 * h);
        const acceleration = (at(seconds + h) - 2 * at(seconds) + at(seconds - h)) / (h * h);
        const damping = Math.min(
          resolved.damping,
          2 * Math.sqrt(resolved.stiffness * resolved.mass),
        );
        const expected = (-resolved.stiffness * at(seconds) - damping * speed) / resolved.mass;
        expect(acceleration).toBeCloseTo(expected, 0);
      }),
    );
  });
});

describe("random", () => {
  it("gives the same number for the same seed, in [0, 1), spread evenly", () => {
    fc.assert(
      fc.property(fc.oneof(fc.string(), fc.integer()), (seed) => {
        const r = random(seed);
        expect(random(seed)).toBe(r);
        expect(r).toBeGreaterThanOrEqual(0);
        expect(r).toBeLessThan(1);
      }),
    );
    const values = Array.from({ length: 4000 }, (_, i) => random(`seed-${i}`));
    const mean = values.reduce((sum, v) => sum + v, 0) / values.length;
    expect(mean).toBeGreaterThan(0.47);
    expect(mean).toBeLessThan(0.53);
    expect(new Set(values).size).toBe(values.length);
  });
});
