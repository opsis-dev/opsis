import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { atWeight, outlineOf, readFont, typefaceOf, wrapLines } from "./outlines";

const bundled = (file: string) =>
  new Uint8Array(
    readFileSync(path.join(import.meta.dirname, "../../../public/fonts/bundled", file)),
  );

describe("letter outlines for 3D text", () => {
  it("reads a compressed variable font and gives each weight its own outlines", async () => {
    const font = await readFont(bundled("inter/inter-latin-wght-normal.woff2"));
    const regular = typefaceOf(atWeight(font, 400));
    const heavy = typefaceOf(atWeight(font, 800));
    const a = regular.glyphs.A;
    expect(a?.o?.startsWith("m ")).toBe(true);
    expect(a?.ha).toBeGreaterThan(0);
    expect(regular.glyphs[" "]?.ha).toBeGreaterThan(0);
    expect(heavy.glyphs.A?.ha).toBeGreaterThan(a?.ha ?? 0);
    expect(heavy.glyphs.A?.o).not.toBe(a?.o);
    expect(regular.resolution).toBe(font.unitsPerEm);
    expect(Object.keys(regular.glyphs).length).toBeGreaterThan(150);
  });

  it("keeps the weight a font without a weight axis has", async () => {
    const font = await readFont(
      bundled("instrument-serif/instrument-serif-latin-400-normal.woff2"),
    );
    expect(atWeight(font, 900)).toBe(font);
  });

  it("writes curves end point first, as the outline format reads them", () => {
    expect(
      outlineOf([
        { command: "moveTo", args: [0, 0] },
        { command: "quadraticCurveTo", args: [5, 10, 20, 0] },
        { command: "bezierCurveTo", args: [1, 2, 3, 4, 5.555, 6] },
        { command: "closePath", args: [] },
      ]),
    ).toBe("m 0 0 q 20 0 5 10 b 5.56 6 1 2 3 4");
  });
});

describe("wrapping 3D text", () => {
  const widthOf = (s: string) => s.length;

  it("breaks on spaces once a line gets too wide", () => {
    expect(wrapLines("Ship faster with demos that move", 12, widthOf)).toEqual([
      "Ship faster",
      "with demos",
      "that move",
    ]);
  });

  it("keeps a word longer than the line whole, and keeps the user's own line breaks", () => {
    expect(wrapLines("Extraordinarily big\nwin", 5, widthOf)).toEqual([
      "Extraordinarily",
      "big",
      "win",
    ]);
  });
});
