import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import {
  drawn,
  kindOf,
  MARK_KINDS,
  moved,
  pointsOf,
  stepNumbers,
  twoEnded,
  withEnd,
} from "./marks";

const unit = fc.integer({ min: 0, max: 1000 }).map((n) => n / 1000);
const anywhere = fc.double({ min: -2, max: 3, noNaN: true });
const points = fc.record({ x: unit, y: unit, x2: unit, y2: unit });
const point = fc.record({ x: anywhere, y: anywhere });

const onPicture = (p: { x: number; y: number; x2: number; y2: number }) =>
  [p.x, p.y, p.x2, p.y2].every((v) => v >= 0 && v <= 1 && Math.round(v * 1000) === v * 1000);

describe("marks", () => {
  it("moves a mark as a whole as far as the picture lets it", () => {
    fc.assert(
      fc.property(points, anywhere, anywhere, (p, dx, dy) => {
        const m = moved(p, dx, dy);
        expect(onPicture(m)).toBe(true);
        expect(m.x2 - m.x).toBeCloseTo(p.x2 - p.x, 2);
        expect(m.y2 - m.y).toBeCloseTo(p.y2 - p.y, 2);
        const left = Math.min(p.x, p.x2);
        const right = Math.max(p.x, p.x2);
        const shiftX = m.x - p.x;
        const fits = dx >= -left && dx <= 1 - right;
        const atEdge = Math.min(m.x, m.x2) < 0.001 || Math.max(m.x, m.x2) > 0.999;
        expect(fits ? Math.abs(shiftX - dx) < 0.005 : atEdge).toBe(true);
      }),
    );
  });

  it("drags only the end it holds and keeps it on the picture", () => {
    fc.assert(
      fc.property(points, fc.constantFrom(0 as const, 1 as const), point, (p, end, at) => {
        const m = withEnd(p, end, at);
        expect(onPicture(m)).toBe(true);
        const other =
          end === 0
            ? [
                [m.x2, m.y2],
                [p.x2, p.y2],
              ]
            : [
                [m.x, m.y],
                [p.x, p.y],
              ];
        expect(other[0]).toEqual(other[1]);
        const held = end === 0 ? [m.x, m.y] : [m.x2, m.y2];
        expect(held[0]).toBeCloseTo(Math.min(1, Math.max(0, at.x)), 3);
        expect(held[1]).toBeCloseTo(Math.min(1, Math.max(0, at.y)), 3);
      }),
    );
  });

  it("draws two-ended marks from a drag long enough and places steps and text on a click", () => {
    fc.assert(
      fc.property(fc.constantFrom(...MARK_KINDS), point, point, (kind, start, end) => {
        const mark = drawn(kind, start, end);
        const long = Math.hypot(end.x - start.x, end.y - start.y) >= 0.01;
        expect(mark !== null).toBe(!twoEnded(kind) || long);
        expect(mark === null || onPicture(mark)).toBe(true);
        expect(twoEnded(kind) || (mark?.x === mark?.x2 && mark?.y === mark?.y2)).toBe(true);
      }),
    );
  });

  it("numbers steps in list order and reads stored points", () => {
    fc.assert(
      fc.property(
        fc.array(fc.record({ kind: fc.constantFrom(...MARK_KINDS, "nonsense") }), {
          maxLength: 12,
        }),
        (marks) => {
          const numbers = stepNumbers(marks);
          const steps = numbers.filter((n) => n !== undefined);
          expect(steps).toEqual(steps.map((_, i) => i + 1));
          marks.forEach((m, i) => expect(numbers[i] !== undefined).toBe(kindOf(m) === "step"));
        },
      ),
    );
    expect(kindOf({ kind: "nonsense" })).toBe("arrow");
    expect(pointsOf({ x: 0.5, y: "no" })).toEqual({ x: 0.5, y: 0, x2: 0, y2: 0 });
  });
});
