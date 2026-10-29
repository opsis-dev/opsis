import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { laneCentres, stepPositions, type LaneBox, type LaneEdge } from "./edge-lanes";

const COLUMN = 420;
const WIDTH = 256;

const graph = fc
  .array(fc.array(fc.integer({ min: 60, max: 400 }), { minLength: 1, maxLength: 5 }), {
    minLength: 2,
    maxLength: 4,
  })
  .chain((columns) => {
    const boxes = new Map<string, LaneBox>();
    columns.forEach((heights, c) => {
      let y = 0;
      heights.forEach((height, r) => {
        boxes.set(`${c}:${r}`, { x: c * COLUMN, y, width: WIDTH, height });
        y += height + 32;
      });
    });
    const ids = [...boxes.keys()];
    const edge = fc
      .tuple(fc.constantFrom(...ids), fc.constantFrom(...ids))
      .filter(([a, b]) => Number(a.split(":")[0]) < Number(b.split(":")[0]));
    return fc.array(edge, { maxLength: 14 }).map((pairs) => ({
      boxes,
      edges: pairs.map(([source, target], i): LaneEdge => ({ id: `e${i}`, source, target })),
    }));
  });

const span = (boxes: ReadonlyMap<string, LaneBox>, e: LaneEdge) => {
  const a = boxes.get(e.source);
  const b = boxes.get(e.target);
  if (!a || !b) throw new Error("missing box");
  return { top: Math.min(a.y, b.y), bottom: Math.max(a.y + a.height, b.y + b.height), column: b.x };
};

describe("edge lanes", () => {
  it("keeps every turn inside the gap between the two cards", () => {
    fc.assert(
      fc.property(graph, ({ boxes, edges }) => {
        for (const { x, left, right } of laneCentres(boxes, edges).values()) {
          expect(x).toBeGreaterThanOrEqual(left);
          expect(x).toBeLessThanOrEqual(right);
        }
        for (const step of stepPositions(boxes, edges).values()) {
          expect(step).toBeGreaterThanOrEqual(0);
          expect(step).toBeLessThanOrEqual(1);
        }
      }),
    );
  });

  it("never runs two edges into the same column down the same line where they overlap", () => {
    fc.assert(
      fc.property(graph, ({ boxes, edges }) => {
        const centres = laneCentres(boxes, edges);
        for (const a of edges) {
          for (const b of edges) {
            if (a.id >= b.id) continue;
            const sa = span(boxes, a);
            const sb = span(boxes, b);
            const overlap = sa.top <= sb.bottom && sb.top <= sa.bottom;
            const ca = centres.get(a.id);
            const cb = centres.get(b.id);
            if (sa.column !== sb.column || !overlap || !ca || !cb) continue;
            expect(Math.abs(ca.x - cb.x)).toBeGreaterThan(0.5);
          }
        }
      }),
    );
  });

  it("gives each of three clips entering a sequence its own turn", () => {
    const boxes = new Map<string, LaneBox>([
      ["a", { x: 0, y: 0, width: WIDTH, height: 120 }],
      ["b", { x: 0, y: 160, width: WIDTH, height: 120 }],
      ["c", { x: 0, y: 320, width: WIDTH, height: 120 }],
      ["seq", { x: COLUMN, y: 140, width: WIDTH, height: 200 }],
    ]);
    const edges = ["a", "b", "c"].map((source) => ({ id: source, source, target: "seq" }));
    expect(new Set(stepPositions(boxes, edges).values()).size).toBe(3);
  });
});
