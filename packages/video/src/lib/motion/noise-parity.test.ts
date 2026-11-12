import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, it } from "vitest";
import { noiseField } from "#video/lib/motion/noise.ts";

const FIXTURE = path.resolve(import.meta.dirname, "../../../../../fixtures/scene/noise-field.json");

const SEEDS = ["noise:0", "noise:7", "shapes", ""];

const POINTS: [number, number, number][] = [
  [0, 0, 0],
  [0.5, 0.25, 0.125],
  [1.7, -2.3, 0.9],
  [-12.5, 7.25, 3.5],
  [100.1, 200.2, 300.3],
  [0.004, 731, 0.12],
  [-0.001, -0.002, -0.003],
  [3, 3, 3],
  [255.5, 256.5, -257.5],
  [1e-9, 2e-9, 3e-9],
];

function recorded() {
  return SEEDS.map((seed) => {
    const field = noiseField(seed);
    return { seed, points: POINTS, values: POINTS.map(([x, y, z]) => field(x, y, z)) };
  });
}

it("records what the film's noise field gives, for the native renderer's port to match", () => {
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
