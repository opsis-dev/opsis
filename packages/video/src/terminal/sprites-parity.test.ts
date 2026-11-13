import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, it } from "vitest";
import { isSprite, type SpriteCell, type SpriteShape, spriteFor } from "#video/terminal/sprites.ts";

const FIXTURE = path.resolve(
  import.meta.dirname,
  "../../../../fixtures/scene/terminal-sprites.json",
);

const CELLS: SpriteCell[] = [
  { width: 8.4, height: 17, thickness: 1 },
  { width: 13, height: 28, thickness: 1 },
  { width: 28.8, height: 60, thickness: 3 },
];

const RANGES: Array<[number, number]> = [
  [0x2500, 0x2600],
  [0x2800, 0x2900],
  [0xe0b0, 0xe0e0],
  [0xf5d0, 0xf610],
  [0x1cc00, 0x1cf00],
  [0x1fb00, 0x1fc00],
];

type Shown =
  | { kind: "rect"; x: number; y: number; width: number; height: number; opacity: number }
  | { kind: "fill"; d: string; opacity: number; clip: boolean }
  | { kind: "stroke"; d: string; width: number; clip: boolean; inner: boolean }
  | { kind: "inverted"; shapes: Shown[] };

function shown(shape: SpriteShape): Shown {
  if (shape.kind === "rect") return shape;
  if (shape.kind === "fill")
    return { kind: "fill", d: shape.d, opacity: shape.opacity ?? 1, clip: shape.clip === true };
  if (shape.kind === "stroke")
    return {
      kind: "stroke",
      d: shape.d,
      width: shape.width,
      clip: shape.clip === true,
      inner: shape.inner === true,
    };
  return { kind: "inverted", shapes: shape.shapes.map(shown) };
}

function recorded() {
  const codes = RANGES.flatMap(([from, to]) =>
    Array.from({ length: to - from }, (_, i) => from + i).filter(isSprite),
  );
  return {
    cells: CELLS.map((cell) => ({
      cell,
      sprites: codes.map((code) => ({ code, shapes: spriteFor(code, cell)?.map(shown) ?? null })),
    })),
  };
}

it("records the terminal's sprites, for the native renderer's port to match", () => {
  const made = JSON.parse(JSON.stringify(recorded()));
  if (process.env.OPSIS_WRITE_GOLDEN === "1") {
    mkdirSync(path.dirname(FIXTURE), { recursive: true });
    writeFileSync(FIXTURE, `${JSON.stringify(made)}\n`);
  }
  expect(existsSync(FIXTURE), `${FIXTURE} is missing: record it with OPSIS_WRITE_GOLDEN=1`).toBe(
    true,
  );
  expect(JSON.parse(readFileSync(FIXTURE, "utf8"))).toEqual(made);
});
