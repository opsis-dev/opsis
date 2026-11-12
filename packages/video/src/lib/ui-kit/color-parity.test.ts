import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, it } from "vitest";
import { interpolateColors, mixOklch } from "#video/lib/ui-kit/color.ts";

const FIXTURE = path.resolve(import.meta.dirname, "../../../../../fixtures/scene/oklch-mix.json");

const COLORS = [
  "#09090b",
  "#fafafa",
  "#ffffff",
  "#000000",
  "#7c3aed",
  "#f472b6",
  "#22c55e",
  "#0ea5e9",
  "#e8b86d",
  "transparent",
  "rgba(255, 0, 0, 0.5)",
  "oklch(0.7 0.25 30)",
  "oklch(0.9 0.37 145)",
  "hsl(210 80% 40%)",
  "rebeccapurple",
];

const STEPS = [0, 0.1, 0.25, 0.5, 0.75, 0.9, 1];

function recorded() {
  const mixes = COLORS.flatMap((a, i) =>
    COLORS.filter((_, j) => j !== i && (i + j) % 3 === 0).map((b) => ({
      a,
      b,
      values: STEPS.map((t) => mixOklch(a, b, t)),
    })),
  );
  const ramps = [
    { range: [0, 1], colors: ["#09090b", "#7c3aed"] },
    { range: [0, 0.5, 1], colors: ["#22c55e", "#e8b86d", "#f472b6"] },
    { range: [10, 20, 40], colors: ["#000000", "#ffffff", "#0ea5e9"] },
  ].map((ramp) => ({
    ...ramp,
    at: [-1, 0, 0.3, 0.5, 0.8, 1, 15, 30, 50],
    values: [-1, 0, 0.3, 0.5, 0.8, 1, 15, 30, 50].map((x) =>
      interpolateColors(x, ramp.range, ramp.colors),
    ),
  }));
  return { mixes, ramps };
}

it("records what the film's colour mixing gives, for the native renderer's port to match", () => {
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
