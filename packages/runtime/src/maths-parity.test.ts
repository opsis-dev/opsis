import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, it } from "vitest";
import { Easing, type EasingFunction } from "#runtime/easing.ts";
import { type InterpolateOptions, interpolate } from "#runtime/interpolate.ts";
import { random } from "#runtime/random.ts";
import {
  measureSpring,
  spring,
  type SpringConfig,
  springConfig,
  springProgress,
} from "#runtime/spring.ts";

const FIXTURE = path.resolve(import.meta.dirname, "../../../fixtures/scene/runtime-maths.json");

const SEEDS: (number | string)[] = [
  0,
  1,
  2,
  3,
  7,
  42,
  0.1,
  0.5,
  1.5,
  -3,
  -0.25,
  1e-7,
  1e21,
  2.5e22,
  123456789,
  0.30000000000000004,
  "",
  "a",
  "seed-1",
  "ünïcode",
  "🙂",
  "particle-12",
];

const SPRINGS: Partial<SpringConfig>[] = [
  {},
  { damping: 200 },
  { mass: 2, stiffness: 50, damping: 5 },
  { overshootClamping: true },
  { damping: 20, stiffness: 180 },
  { damping: 12, mass: 0.6, stiffness: 120 },
];

const SECONDS = [0, 0.01, 0.1, 0.25, 0.5, 1, 2, 5];
const STEPS = [0, 0.05, 0.1, 0.25, 0.4, 0.5, 0.6, 0.75, 0.9, 0.95, 1];

type Named = { name: string; params: number[]; inner?: string };

const EASINGS: Named[] = [
  { name: "linear", params: [] },
  { name: "ease", params: [] },
  { name: "quad", params: [] },
  { name: "cubic", params: [] },
  { name: "poly", params: [4] },
  { name: "sin", params: [] },
  { name: "circle", params: [] },
  { name: "exp", params: [] },
  { name: "elastic", params: [1] },
  { name: "elastic", params: [2.5] },
  { name: "back", params: [1.70158] },
  { name: "bounce", params: [] },
  { name: "bezier", params: [0.16, 1, 0.3, 1] },
  { name: "bezier", params: [0.7, 0, 0.84, 0] },
  { name: "bezier", params: [0.34, 1.56, 0.64, 1] },
  { name: "out", params: [], inner: "cubic" },
  { name: "inOut", params: [], inner: "quad" },
  { name: "inOut", params: [], inner: "exp" },
  { name: "spring", params: [1, 12, 120] },
];

function easingOf({ name, params, inner }: Named): EasingFunction {
  const [a = 0, b = 0, c = 0, d = 0] = params;
  const base = inner ? easingOf({ name: inner, params: [] }) : Easing.linear;
  switch (name) {
    case "poly":
      return Easing.poly(a);
    case "elastic":
      return Easing.elastic(a);
    case "back":
      return Easing.back(a);
    case "bezier":
      return Easing.bezier(a, b, c, d);
    case "out":
      return Easing.out(base);
    case "inOut":
      return Easing.inOut(base);
    case "spring":
      return Easing.spring({ mass: a, damping: b, stiffness: c });
    default:
      return Easing[name as "linear"];
  }
}

const INTERPOLATIONS: {
  input: number[];
  output: number[];
  options: InterpolateOptions & { easingName?: string };
  at: number[];
}[] = [
  { input: [0, 1], output: [0, 100], options: {}, at: [-0.5, 0, 0.3, 1, 1.5] },
  {
    input: [0, 10, 30],
    output: [1, 0.2, 5],
    options: { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
    at: [-5, 0, 5, 10, 20, 30, 40],
  },
  { input: [2, 6], output: [-3, 9], options: { extrapolateRight: "wrap" }, at: [1, 7, 11.5] },
  {
    input: [0, 1],
    output: [0.2, 3],
    options: { output: "perceptual-scale" },
    at: [0, 0.25, 0.5, 1],
  },
  {
    input: [0, 1],
    output: [0, 1],
    options: { extrapolateLeft: "identity", extrapolateRight: "identity" },
    at: [-2, 0.5, 3],
  },
  {
    input: [0, 20],
    output: [10, 0],
    options: { easingName: "ease", extrapolateRight: "clamp" },
    at: [0, 3, 10, 19.5, 25],
  },
];

function recorded() {
  const randoms = SEEDS.map((seed) => ({ seed, value: random(seed) }));
  const springs = SPRINGS.map((config) => ({
    config: springConfig(config),
    progress: SECONDS.map((s) => springProgress(springConfig(config), s)),
    rest30: measureSpring({ fps: 30, config }),
    rest60: measureSpring({ fps: 60, config }),
    stretched: [0, 3, 10, 17, 25, 40].map((frame) =>
      spring({ frame, fps: 30, config, durationInFrames: 24, from: -5, to: 5 }),
    ),
    delayed: [0, 5, 12, 30].map((frame) => spring({ frame, fps: 60, config, delay: 6 })),
  }));
  const easings = EASINGS.map((easing) => ({
    ...easing,
    values: STEPS.map((t) => easingOf(easing)(t)),
  }));
  const interpolations = INTERPOLATIONS.map(({ input, output, options, at }) => {
    const { easingName, ...rest } = options;
    const easing = easingName ? easingOf({ name: easingName, params: [] }) : undefined;
    return {
      input,
      output,
      options,
      at,
      values: at.map((x) => interpolate(x, input, output, { ...rest, easing })),
    };
  });
  return { randoms, springs, easings, interpolations };
}

it("records what the film's maths gives, for the native renderer's ports to match", () => {
  const made = recorded();
  if (process.env.OPSIS_WRITE_GOLDEN === "1") {
    mkdirSync(path.dirname(FIXTURE), { recursive: true });
    writeFileSync(FIXTURE, `${JSON.stringify(made, null, 1)}\n`);
  }
  expect(existsSync(FIXTURE), `${FIXTURE} is missing: record it with OPSIS_WRITE_GOLDEN=1`).toBe(
    true,
  );
  expect(JSON.parse(readFileSync(FIXTURE, "utf8"))).toEqual(JSON.parse(JSON.stringify(made)));
});
