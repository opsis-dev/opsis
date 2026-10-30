import * as fc from "fast-check";
import { expect, it } from "vitest";
import { type MarkLook, markPaths, strokeShown } from "#video/core/hand-drawn-marks.ts";

const LOOKS: MarkLook[] = ["box", "circle", "underline", "crossed-off", "marker"];

const shape = fc.record({
  look: fc.constantFrom(...LOOKS),
  seed: fc.integer({ min: 1, max: 1_000_000 }),
  strokes: fc.integer({ min: 1, max: 6 }),
  roughness: fc.option(fc.double({ min: 0, max: 4, noNaN: true }), { nil: undefined }),
});

const box = fc.record({
  x: fc.double({ min: -500, max: 500, noNaN: true }),
  y: fc.double({ min: -500, max: 500, noNaN: true }),
  w: fc.double({ min: 1, max: 2000, noNaN: true }),
  h: fc.double({ min: 1, max: 2000, noNaN: true }),
});

const PATHS_PER_STROKE: Record<MarkLook, number> = {
  box: 4,
  circle: 1,
  underline: 1,
  "crossed-off": 2,
  marker: 1,
};

it("draws the same strokes for the same seed, one set per stroke", () => {
  fc.assert(
    fc.property(shape, box, (mark, area) => {
      const drawn = markPaths(mark, area);
      expect(markPaths(mark, area)).toEqual(drawn);
      expect(drawn.paths).toHaveLength(mark.strokes * PATHS_PER_STROKE[mark.look]);
      expect(drawn.paths.every((d) => d.startsWith("M"))).toBe(true);
      expect(drawn.strokeWidth).toBe(mark.look === "marker" ? area.h : undefined);
    }),
  );
});

it("draws every stroke of a box or circle with its own wobble", () => {
  fc.assert(
    fc.property(
      fc.constantFrom<MarkLook>("box", "circle"),
      fc.integer({ min: 1, max: 1_000_000 }),
      fc.integer({ min: 2, max: 6 }),
      fc.double({ min: 0.5, max: 4, noNaN: true }),
      box,
      (look, seed, strokes, roughness, area) => {
        const { paths } = markPaths({ look, seed, strokes, roughness }, area);
        const each = PATHS_PER_STROKE[look];
        const strokeTexts = Array.from({ length: strokes }, (_, i) =>
          paths.slice(i * each, (i + 1) * each).join(" "),
        );
        expect(new Set(strokeTexts).size).toBe(strokes);
      },
    ),
  );
});

it("draws nothing around an empty box", () => {
  fc.assert(
    fc.property(shape, box, fc.boolean(), (mark, area, flat) => {
      expect(markPaths(mark, flat ? { ...area, h: 0 } : { ...area, w: 0 }).paths).toEqual([]);
    }),
  );
});

it("draws the strokes one after another as progress goes from 0 to 1", () => {
  fc.assert(
    fc.property(
      fc.integer({ min: 1, max: 40 }),
      fc.double({ min: 0, max: 1, noNaN: true }),
      fc.double({ min: 0, max: 1, noNaN: true }),
      (strokes, a, b) => {
        const [early, late] = a < b ? [a, b] : [b, a];
        function shown(progress: number) {
          return Array.from({ length: strokes }, (_, i) => strokeShown(progress, i, strokes));
        }
        expect(shown(0).every((part) => part === 0)).toBe(true);
        expect(shown(1).every((part) => part === 1)).toBe(true);
        expect(shown(early).reduce((sum, part) => sum + part, 0)).toBeCloseTo(early * strokes, 9);
        expect(shown(early).every((part, i) => part <= (shown(late)[i] ?? 0))).toBe(true);
        expect(shown(late).every((part, i) => part === 0 || (shown(late)[i - 1] ?? 1) === 1)).toBe(
          true,
        );
      },
    ),
  );
});
