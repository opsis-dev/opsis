import { resampleIcon } from "morphicons";
import { describe, expect, it } from "vitest";
import { usedGlyphs } from "@/nodes/looks";
import { catalogNodes } from "@/store/catalog";
import { NODE_GLYPHS } from "./node-glyphs.generated";

const HALF_STROKE = 1;
const SAMPLES_PER_SUBPATH = 160;

type Drawing = { points: number[]; byCell: Map<string, number[]> };

function cellKey(x: number, y: number) {
  return `${Math.floor(x / HALF_STROKE)},${Math.floor(y / HALF_STROKE)}`;
}

function drawingOf(glyph: string): Drawing {
  const points = resampleIcon(NODE_GLYPHS[glyph], SAMPLES_PER_SUBPATH).flatMap(({ pts }) => [
    ...pts,
  ]);
  const byCell = new Map<string, number[]>();
  for (let i = 0; i < points.length; i += 2) {
    const key = cellKey(points[i], points[i + 1]);
    byCell.set(key, [...(byCell.get(key) ?? []), points[i], points[i + 1]]);
  }
  return { points, byCell };
}

function hasPointNear(drawing: Drawing, x: number, y: number): boolean {
  const [cx, cy] = [Math.floor(x / HALF_STROKE), Math.floor(y / HALF_STROKE)];
  for (let dy = -1; dy <= 1; dy++) {
    for (let dx = -1; dx <= 1; dx++) {
      const near = drawing.byCell.get(`${cx + dx},${cy + dy}`) ?? [];
      for (let j = 0; j < near.length; j += 2) {
        if ((near[j] - x) ** 2 + (near[j + 1] - y) ** 2 <= HALF_STROKE ** 2) return true;
      }
    }
  }
  return false;
}

function liesOn(drawing: Drawing, other: Drawing): boolean {
  for (let i = 0; i < drawing.points.length; i += 2) {
    if (!hasPointNear(other, drawing.points[i], drawing.points[i + 1])) return false;
  }
  return true;
}

function sameDrawing(a: Drawing, b: Drawing) {
  return liesOn(a, b) && liesOn(b, a);
}

describe("node glyphs", () => {
  it("holds exactly the glyphs the node looks use (run bun run glyphs:bundle after changing them)", () => {
    const looks = catalogNodes().flatMap((node) => (node.look ? [node.look] : []));
    const used: ReadonlySet<string> = new Set(usedGlyphs(looks));
    expect([...used].filter((glyph) => !NODE_GLYPHS[glyph])).toEqual([]);
    expect(Object.keys(NODE_GLYPHS).filter((glyph) => !used.has(glyph))).toEqual([]);
  });

  it("draws every glyph with strokes only", () => {
    const filled = Object.entries(NODE_GLYPHS).filter(([, node]) =>
      node.some(([, attrs]) => attrs.fill !== undefined && attrs.fill !== "none"),
    );
    expect(filled.map(([glyph]) => glyph)).toEqual([]);
  });

  it("never gives nodes that move alike two names for the same drawing", () => {
    const looked = catalogNodes().flatMap((node) => (node.look ? [{ ...node.look, node }] : []));
    const drawings = new Map(Object.keys(NODE_GLYPHS).map((glyph) => [glyph, drawingOf(glyph)]));
    function moves(look: (typeof looked)[number]) {
      return look.motion ?? look.morph ?? "still";
    }
    const clashes: string[] = [];
    for (const [i, a] of looked.entries()) {
      for (const b of looked.slice(i + 1)) {
        if (a.glyph === b.glyph || moves(a) !== moves(b)) continue;
        const [da, db] = [drawings.get(a.glyph), drawings.get(b.glyph)];
        if (da && db && sameDrawing(da, db)) {
          clashes.push(`${a.node.title} (${a.glyph}) and ${b.node.title} (${b.glyph})`);
        }
      }
    }
    expect(clashes).toEqual([]);
  });
});
