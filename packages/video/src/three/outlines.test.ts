import { describe, expect, it } from "vitest";
import { wrapLines } from "#video/three/outlines.ts";

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
