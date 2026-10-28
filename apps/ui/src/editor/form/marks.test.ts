import { describe, expect, it } from "vitest";
import { drawn, kindOf, moved, pointsOf, stepNumbers, withEnd } from "./marks";

describe("marks", () => {
  it("moves a mark as a whole and keeps it inside the picture", () => {
    const points = { x: 0.1, y: 0.2, x2: 0.3, y2: 0.4 };
    expect(moved(points, 0.1, 0.1)).toEqual({ x: 0.2, y: 0.3, x2: 0.4, y2: 0.5 });
    expect(moved(points, -0.5, 0.9)).toEqual({ x: 0, y: 0.8, x2: 0.2, y2: 1 });
    expect(moved({ x: 0.9, y: 0.5, x2: 0.7, y2: 0.5 }, 0.5, 0)).toEqual({
      x: 1,
      y: 0.5,
      x2: 0.8,
      y2: 0.5,
    });
  });

  it("drags one end of an arrow or a corner of a box", () => {
    const points = { x: 0.1, y: 0.2, x2: 0.3, y2: 0.4 };
    expect(withEnd(points, 0, { x: 0.05, y: -1 })).toEqual({ x: 0.05, y: 0, x2: 0.3, y2: 0.4 });
    expect(withEnd(points, 1, { x: 0.6666, y: 0.5 })).toEqual({
      x: 0.1,
      y: 0.2,
      x2: 0.667,
      y2: 0.5,
    });
  });

  it("draws two-ended marks from a drag and places steps and text on a click", () => {
    const start = { x: 0.2, y: 0.2 };
    expect(drawn("arrow", start, { x: 0.5, y: 0.6 })).toEqual({ x: 0.2, y: 0.2, x2: 0.5, y2: 0.6 });
    expect(drawn("rectangle", start, { x: 0.205, y: 0.2 })).toBeNull();
    expect(drawn("step", start, { x: 0.205, y: 0.2 })).toEqual({
      x: 0.205,
      y: 0.2,
      x2: 0.205,
      y2: 0.2,
    });
  });

  it("numbers steps in list order and reads stored points", () => {
    const marks = [{ kind: "step" }, { kind: "arrow" }, { kind: "step" }, { kind: "nonsense" }];
    expect(stepNumbers(marks)).toEqual([1, undefined, 2, undefined]);
    expect(kindOf(marks[3] ?? {})).toBe("arrow");
    expect(pointsOf({ x: 0.5, y: "no" })).toEqual({ x: 0.5, y: 0, x2: 0, y2: 0 });
  });
});
