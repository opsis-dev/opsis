import { readFileSync } from "node:fs";
import path from "node:path";
import { resampleIcon } from "morphicons";
import { describe, expect, it } from "vitest";
import { editorCatalog } from "./lib/editor-data";
import { ROOT } from "./lib/tool-config";

type IconNode = ReadonlyArray<readonly [string, Record<string, string>]>;
type Look = { glyph: string; morph?: string; motion?: string };
type CatalogNode = { title: string; look?: Look };
type Drawing = { points: number[]; byCell: Map<string, number[]> };
type Morph = { from: number; step: number; closed: boolean[]; frames: number[][][] };

const HALF_STROKE = 1;
const SAMPLES_PER_SUBPATH = 160;
const DENSE_GAP = 0.25;
const FALLBACK_GLYPH = "lucide:box";
const bundled = JSON.parse(
  readFileSync(path.join(ROOT, "crates", "ui", "glyphs.generated.json"), "utf8"),
) as { glyphs: Record<string, IconNode>; morphs: Record<string, Morph> };
const glyphs = bundled.glyphs;
const morphs = bundled.morphs;
const nodes = (editorCatalog() as { nodes: CatalogNode[] }).nodes;
const looked = nodes.flatMap((node) => (node.look ? [{ ...node.look, node }] : []));

function cellKey(x: number, y: number): string {
  return `${Math.floor(x / HALF_STROKE)},${Math.floor(y / HALF_STROKE)}`;
}

function drawingOf(glyph: string): Drawing {
  return drawingFrom(
    resampleIcon(glyphs[glyph], SAMPLES_PER_SUBPATH).flatMap(({ pts }) => [...pts]),
  );
}

function densePath(line: readonly number[], closed: boolean): number[] {
  const ends = closed ? [...line, line[0], line[1]] : [...line];
  const points: number[] = [];
  for (let i = 0; i + 3 < ends.length; i += 2) {
    const [x0, y0, x1, y1] = [ends[i], ends[i + 1], ends[i + 2], ends[i + 3]];
    const steps = Math.max(1, Math.ceil(Math.hypot(x1 - x0, y1 - y0) / DENSE_GAP));
    for (let k = 0; k < steps; k++)
      points.push(x0 + ((x1 - x0) * k) / steps, y0 + ((y1 - y0) * k) / steps);
  }
  return [...points, ends[ends.length - 2], ends[ends.length - 1]];
}

function morphDrawing(morph: Morph, t: number): Drawing {
  const frame = morph.frames[Math.round((t - morph.from) / morph.step)];
  return drawingFrom(frame.flatMap((line, k) => densePath(line, morph.closed[k])));
}

function drawingFrom(points: number[]): Drawing {
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

function sameDrawing(a: Drawing, b: Drawing): boolean {
  return liesOn(a, b) && liesOn(b, a);
}

function moves(look: Look): string {
  return look.motion ?? look.morph ?? "still";
}

describe("node glyphs", () => {
  it("holds exactly the glyphs the node looks use (run bun run glyphs:bundle after changing them)", () => {
    const used = new Set([
      FALLBACK_GLYPH,
      ...looked.flatMap((look) => [look.glyph, look.morph ?? []].flat()),
    ]);
    expect([...used].filter((glyph) => !glyphs[glyph])).toEqual([]);
    expect(Object.keys(glyphs).filter((glyph) => !used.has(glyph))).toEqual([]);
  });

  it("bakes one morph for each look that morphs (run bun run glyphs:bundle after changing them)", () => {
    const pairs = new Set(
      looked.flatMap((look) => (look.morph ? [`${look.glyph}>${look.morph}`] : [])),
    );
    expect(Object.keys(morphs).toSorted()).toEqual([...pairs].toSorted());
  });

  it("starts each morph on its glyph and ends it on the glyph it turns into", () => {
    const off: string[] = [];
    for (const [pair, morph] of Object.entries(morphs)) {
      const [from, to] = pair.split(">");
      for (const [t, glyph] of [
        [0, from],
        [1, to],
      ] as const) {
        if (!sameDrawing(morphDrawing(morph, t), drawingOf(glyph))) off.push(`${pair} at ${t}`);
      }
    }
    expect(off).toEqual([]);
  });

  it("draws every glyph with strokes only", () => {
    const filled = Object.entries(glyphs).filter(([, node]) =>
      node.some(([, attrs]) => attrs.fill !== undefined && attrs.fill !== "none"),
    );
    expect(filled.map(([glyph]) => glyph)).toEqual([]);
  });

  it("never gives nodes that move alike two names for the same drawing", () => {
    const drawings = new Map(Object.keys(glyphs).map((glyph) => [glyph, drawingOf(glyph)]));
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
