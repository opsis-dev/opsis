import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { formatBytes } from "#video/common/format.ts";

const UNITS = ["B", "KiB", "MiB", "GiB", "TiB"];

describe("formatBytes", () => {
  it("writes sizes in binary units like the cards do", () => {
    expect([formatBytes(512), formatBytes(1536), formatBytes(29 * 1024 * 1024)]).toEqual([
      "512 B",
      "1.5 KiB",
      "29 MiB",
    ]);
  });

  it("writes every size under its unit and within a twentieth of a unit of its bytes", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 40 }).chain((bits) => fc.integer({ min: 0, max: 2 ** bits })),
        (bytes) => {
          const [number, unit] = formatBytes(bytes).split(" ");
          const power = UNITS.indexOf(unit);
          expect(power).toBeGreaterThanOrEqual(0);
          const exact = bytes / 1024 ** power;
          expect(power === 0 ? number === String(bytes) : exact >= 1).toBe(true);
          expect(Math.abs(Number(number) - exact)).toBeLessThanOrEqual(
            power === 0 ? 0 : 0.05 + 1e-9,
          );
        },
      ),
    );
  });
});
