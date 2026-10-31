import * as fc from "fast-check";
import { getPathBBox, parsePathString } from "svg-path-commander/util";
import { expect, it } from "vitest";
import {
  fitToBox,
  MORPH_SHAPES,
  morphPath,
  prepareMorph,
  type MorphShapeName,
} from "#video/elements/transition/transition-morph-shape/path-morph.ts";
import { isNumber } from "#video/common/guards.ts";

const shape = fc.constantFrom(...(Object.keys(MORPH_SHAPES) as MorphShapeName[]));
const area = fc.record({
  x: fc.double({ min: -100, max: 100, noNaN: true }),
  y: fc.double({ min: -100, max: 100, noNaN: true }),
  width: fc.double({ min: 0.5, max: 400, noNaN: true }),
  height: fc.double({ min: 0.5, max: 400, noNaN: true }),
});

function close(a: number, b: number, scale: number) {
  return Math.abs(a - b) <= 1e-6 * Math.max(1, scale);
}

it("fits a shape into the box, centred, keeping its proportions", () => {
  fc.assert(
    fc.property(shape, area, (name, box) => {
      const source = getPathBBox(MORPH_SHAPES[name]);
      const fitted = getPathBBox(fitToBox(MORPH_SHAPES[name], box));
      const scale = Math.max(box.width, box.height);
      expect(close(fitted.cx, box.x + box.width / 2, scale)).toBe(true);
      expect(close(fitted.cy, box.y + box.height / 2, scale)).toBe(true);
      expect(fitted.width).toBeLessThanOrEqual(box.width + 1e-6 * scale);
      expect(fitted.height).toBeLessThanOrEqual(box.height + 1e-6 * scale);
      expect(close(fitted.width, box.width, scale) || close(fitted.height, box.height, scale)).toBe(
        true,
      );
      expect(close(fitted.width / fitted.height, source.width / source.height, source.width)).toBe(
        true,
      );
    }),
  );
});

function withoutPointSegments(d: string, scale: number): string {
  let end = [0, 0];
  return parsePathString(d)
    .flatMap((segment) => {
      const values = segment.slice(1).filter(isNumber);
      const point = end;
      if (values.length >= 2) end = values.slice(-2);
      const still =
        segment[0] === "C" &&
        values.every((v, i) => Math.abs(v - (point[i % 2] ?? 0)) <= 1e-9 * Math.max(1, scale));
      return still ? [] : [`${segment[0]}${values.join(" ")}`];
    })
    .join(" ");
}

it("starts on the first shape and ends on the second", () => {
  fc.assert(
    fc.property(shape, shape, area, (from, to, box) => {
      const pair = prepareMorph(MORPH_SHAPES[from], MORPH_SHAPES[to], box);
      const scale = Math.max(box.width, box.height);
      for (const [progress, name] of [
        [0, from],
        [1, to],
      ] as const) {
        const drawn = getPathBBox(withoutPointSegments(morphPath(progress, pair), scale));
        const expected = getPathBBox(
          withoutPointSegments(fitToBox(MORPH_SHAPES[name], box), scale),
        );
        expect(close(drawn.x, expected.x, scale)).toBe(true);
        expect(close(drawn.y, expected.y, scale)).toBe(true);
        expect(close(drawn.x2, expected.x2, scale)).toBe(true);
        expect(close(drawn.y2, expected.y2, scale)).toBe(true);
      }
    }),
  );
});

it("moves every point a little for a little more progress", () => {
  fc.assert(
    fc.property(shape, shape, fc.double({ min: 0, max: 0.99, noNaN: true }), (from, to, t) => {
      const pair = prepareMorph(MORPH_SHAPES[from], MORPH_SHAPES[to], { width: 100, height: 100 });
      function numbers(d: string) {
        return (d.match(/-?\d+(\.\d+)?(e-?\d+)?/g) ?? []).map(Number);
      }
      const now = numbers(morphPath(t, pair));
      const next = numbers(morphPath(t + 0.01, pair));
      expect(next).toHaveLength(now.length);
      expect(Math.max(...now.map((value, i) => Math.abs(value - (next[i] ?? value))))).toBeLessThan(
        5,
      );
    }),
  );
});
