import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, it } from "vitest";
import { formatAxisValue, formatNumber, groupDigits } from "#video/lib/charts/chart-utils.ts";

const FIXTURE = path.resolve(import.meta.dirname, "../../../../../fixtures/scene/number-text.json");

const VALUES = [
  0, 1, -1, 0.5, 1.005, 2.675, 9.95, 9.99, 10, 12.345, 99.95, 999, 999.5, 999.95, 1000, 1234, 1250,
  9999, 9999.5, 12345, 99999, 123456, 999499, 999950, 999999, 1e6, 1234567, 12.5e6, 999.95e6, 1e9,
  2.5e9, 1e12, 3.33e12, 1e15, -1234, -999950, 0.004, 0.005, 0.015, 123.456789,
];

const GROUPS = ["1234", "1234567", "12", "999999", "abc1234def", "1234.5678", "-1234567", "12,345"];

function recorded() {
  return {
    values: VALUES,
    compact: VALUES.map((v) => formatNumber(v, "compact")),
    full: VALUES.map((v) => formatNumber(v, "full")),
    axis: VALUES.map((v) => formatAxisValue(v)),
    groups: GROUPS,
    grouped: GROUPS.map((g) => groupDigits(g, "comma")),
  };
}

it("records what the film's number formatting gives, for the native renderer's port to match", () => {
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
