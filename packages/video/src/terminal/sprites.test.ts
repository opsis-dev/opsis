import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { isSprite, spriteFor, strokeFor, type SpriteShape } from "#video/terminal/sprites.ts";

const cellArb = fc
  .record({
    fontSize: fc.integer({ min: 8, max: 64 }),
    lineHeight: fc.double({ min: 1, max: 2, noNaN: true }),
  })
  .map(({ fontSize, lineHeight }) => ({
    width: fontSize * 0.6,
    height: Math.round(fontSize * lineHeight),
    thickness: strokeFor(fontSize),
  }));

const RANGES: Array<[number, number]> = [
  [0x2500, 0x259f],
  [0x2800, 0x28ff],
  [0xe0b0, 0xe0bf],
  [0x1fb00, 0x1fb67],
  [0x1fb70, 0x1fb7b],
  [0x1fb82, 0x1fb8b],
  [0x1cd00, 0x1cde5],
];

const spriteArb = fc.oneof(...RANGES.map(([a, b]) => fc.integer({ min: a, max: b })));

function rects(shapes: SpriteShape[]) {
  return shapes.flatMap((s) => (s.kind === "rect" ? [s] : []));
}

describe("sprites", () => {
  it("draws every rectangle of a sprite inside its cell", () => {
    fc.assert(
      fc.property(spriteArb, cellArb, (cp, cell) => {
        const shapes = spriteFor(cp, cell);
        expect(shapes).not.toBeNull();
        for (const r of rects(shapes ?? [])) {
          expect(r.x).toBeGreaterThanOrEqual(-cell.thickness * 2);
          expect(r.y).toBeGreaterThanOrEqual(-cell.thickness * 2);
          expect(r.x + r.width).toBeLessThanOrEqual(cell.width + cell.thickness * 2 + 1e-9);
          expect(r.y + r.height).toBeLessThanOrEqual(cell.height + cell.thickness * 2 + 1e-9);
          expect(r.width).toBeGreaterThanOrEqual(0);
          expect(r.height).toBeGreaterThanOrEqual(0);
        }
      }),
    );
  });

  it("meets the cell edge at its centre with the stroke's width, so lines join across cells", () => {
    fc.assert(
      fc.property(cellArb, (cell) => {
        const horizontal = rects(spriteFor(0x2500, cell) ?? []);
        const vertical = rects(spriteFor(0x2502, cell) ?? []);
        const crossing = rects(spriteFor(0x253c, cell) ?? []);
        const leftArm = horizontal.find((r) => r.x === 0);
        const upArm = vertical.find((r) => r.y === 0);
        expect(leftArm?.height).toBe(cell.thickness);
        expect(leftArm?.y).toBe(Math.floor((cell.height - cell.thickness) / 2));
        expect(upArm?.width).toBe(cell.thickness);
        expect(upArm?.x).toBe(Math.floor((cell.width - cell.thickness) / 2));
        expect(crossing.find((r) => r.x === 0)?.y).toBe(leftArm?.y);
        expect(crossing.find((r) => r.y === 0)?.x).toBe(upArm?.x);
        const rightEdge = horizontal.find((r) => r.x + r.width === cell.width);
        expect(rightEdge?.y).toBe(leftArm?.y);
      }),
    );
  });

  it("leaves letters and other symbols to the fonts", () => {
    for (const ch of "aZ0→✔🚀日") expect(isSprite(ch.codePointAt(0) ?? 0)).toBe(false);
    expect(spriteFor(0x41, { width: 12, height: 24, thickness: 1 })).toBeNull();
  });

  it("fills a full block and none of an empty braille cell", () => {
    const cell = { width: 12, height: 24, thickness: 1 };
    expect(spriteFor(0x2588, cell)).toEqual([
      { kind: "rect", x: 0, y: 0, width: 12, height: 24, opacity: 1 },
    ]);
    expect(spriteFor(0x2800, cell)).toEqual([]);
    expect(spriteFor(0x28ff, cell)).toHaveLength(8);
  });
});
