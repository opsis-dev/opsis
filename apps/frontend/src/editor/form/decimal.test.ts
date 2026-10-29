import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { framesScale, MS_SCALE, parseShown, shownText, unitScale } from "./decimal";

const RATES = [12, 24, 25, 30, 48, 50, 60, 120];

describe("seconds conversions", () => {
  it("shows stored milliseconds and frames in seconds", () => {
    expect(shownText(1500, MS_SCALE)).toBe("1.5");
    expect(shownText(45, framesScale(30))).toBe("1.5");
  });

  it("reads back every stored time it shows, typed with a point or a comma", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 10_000_000 }),
        fc.constantFrom(...RATES),
        fc.boolean(),
        fc.boolean(),
        (stored, fps, frames, comma) => {
          const scale = frames ? framesScale(fps) : MS_SCALE;
          const shown = shownText(stored, scale);
          expect(parseShown(comma ? shown.replace(".", ",") : shown, scale)).toBe(stored);
        },
      ),
    );
  });

  it("rounds any typed seconds to the nearest whole stored unit", () => {
    fc.assert(
      fc.property(
        fc.double({ min: 0, max: 10_000, noNaN: true }),
        fc.constantFrom(...RATES),
        (seconds, fps) => {
          const stored = parseShown(String(seconds), framesScale(fps));
          expect(stored).not.toBeNull();
          expect(Number.isInteger(stored)).toBe(true);
          expect(Math.abs((stored ?? 0) - seconds * fps)).toBeLessThanOrEqual(0.5);
        },
      ),
    );
    expect(parseShown("", MS_SCALE)).toBeNull();
  });

  it("keeps other units unscaled and unrounded", () => {
    fc.assert(
      fc.property(
        fc.double({ min: -1e6, max: 1e6, noNaN: true }).map((v) => v + 0),
        (value) => {
          expect(parseShown(String(value), unitScale("px", 30))).toBe(value);
        },
      ),
    );
  });
});
