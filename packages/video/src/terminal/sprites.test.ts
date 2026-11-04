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
  [0x25e2, 0x25e5],
  [0x25f8, 0x25fa],
  [0x25ff, 0x25ff],
  [0x2800, 0x28ff],
  [0xe0b0, 0xe0bf],
  [0xe0d2, 0xe0d2],
  [0xe0d4, 0xe0d4],
  [0xf5d0, 0xf60d],
  [0x1fb00, 0x1fbaf],
  [0x1fbbd, 0x1fbbf],
  [0x1fbce, 0x1fbef],
  [0x1cc1b, 0x1cc1e],
  [0x1cc21, 0x1cc3f],
  [0x1cd00, 0x1cde5],
  [0x1ce00, 0x1ce01],
  [0x1ce0b, 0x1ce0c],
  [0x1ce16, 0x1ce19],
  [0x1ce51, 0x1ceaf],
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

  it("draws exactly the codepoints Ghostty draws itself", () => {
    function drawn(cp: number) {
      return RANGES.some(([a, b]) => cp >= a && cp <= b);
    }
    for (let cp = 0x2500; cp <= 0x28ff; cp++) expect(isSprite(cp)).toBe(drawn(cp));
    for (let cp = 0xe0a0; cp <= 0xe0df; cp++) expect(isSprite(cp)).toBe(drawn(cp));
    for (let cp = 0xf5c0; cp <= 0xf61f; cp++) expect(isSprite(cp)).toBe(drawn(cp));
    for (let cp = 0x1cc00; cp <= 0x1cebf; cp++) expect(isSprite(cp)).toBe(drawn(cp));
    for (let cp = 0x1fb00; cp <= 0x1fbff; cp++) expect(isSprite(cp)).toBe(drawn(cp));
  });

  it("keeps what an inverted sprite cuts out, and the parts of a circle that leave the cell, to the cell", () => {
    fc.assert(
      fc.property(spriteArb, cellArb, (cp, cell) => {
        const inverted = (spriteFor(cp, cell) ?? []).flatMap((s) =>
          s.kind === "inverted" ? [s.shapes] : [],
        );
        expect(inverted.every((inner) => inner.length > 0)).toBe(true);
        expect(inverted.flat().every((s) => s.kind !== "inverted")).toBe(true);
      }),
    );
    const cell = { width: 12, height: 24, thickness: 2 };
    for (const cp of [0x1fbe0, 0x1fbec, 0x1cc30, 0x1ce00, 0x1fb98])
      expect(
        (spriteFor(cp, cell) ?? []).every(
          (s) => s.kind !== "rect" && s.kind !== "inverted" && s.clip,
        ),
      ).toBe(true);
  });

  it("keeps separated blocks apart and inside their cell", () => {
    fc.assert(
      fc.property(
        cellArb,
        fc.integer({ min: 0x1cc21, max: 0x1cc2f }),
        fc.integer({ min: 0x1ce51, max: 0x1ce8f }),
        (cell, quad, sext) => {
          for (const cp of [quad, sext]) {
            const parts = rects(spriteFor(cp, cell) ?? []);
            expect(parts.length).toBeGreaterThan(0);
            for (const [i, a] of parts.entries()) {
              expect(a.x).toBeGreaterThan(0);
              expect(a.y).toBeGreaterThan(0);
              expect(a.x + a.width).toBeLessThan(cell.width);
              expect(a.y + a.height).toBeLessThan(cell.height);
              for (const b of parts.slice(i + 1)) {
                const apart =
                  a.x + a.width < b.x ||
                  b.x + b.width < a.x ||
                  a.y + a.height < b.y ||
                  b.y + b.height < a.y;
                expect(apart).toBe(true);
              }
            }
          }
        },
      ),
    );
  });

  it("joins a branch node's arms to box lines at the cell's edges", () => {
    fc.assert(
      fc.property(cellArb, (cell) => {
        const vertical = rects(spriteFor(0x2502, cell) ?? []).find((r) => r.y === 0);
        const horizontal = rects(spriteFor(0x2500, cell) ?? []).find((r) => r.x === 0);
        const node = rects(spriteFor(0xf60c, cell) ?? []);
        expect(node.find((r) => r.y === 0)?.x).toBe(vertical?.x);
        expect(node.find((r) => r.x === 0)?.y).toBe(horizontal?.y);
      }),
    );
  });

  it("fades a fading branch line from full to nothing across the cell", () => {
    const cell = { width: 12, height: 24, thickness: 2 };
    const fading = rects(spriteFor(0xf5d2, cell) ?? []);
    expect(fading[0]?.opacity).toBe(1);
    const opacities = fading.map((r) => r.opacity);
    expect(opacities).toEqual([...opacities].sort((a, b) => b - a));
    expect(rects(spriteFor(0xf5d5, cell) ?? []).map((r) => r.opacity)).toEqual(
      rects(spriteFor(0xf5d5, cell) ?? [])
        .map((r) => r.opacity)
        .sort((a, b) => a - b),
    );
  });
});
