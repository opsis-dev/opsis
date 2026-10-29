import { describe, expect, it } from "vitest";
import { usedGlyphs } from "@/nodes/looks";
import { catalogNodes } from "@/store/catalog";
import { NODE_GLYPHS } from "./node-glyphs.generated";

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
});
