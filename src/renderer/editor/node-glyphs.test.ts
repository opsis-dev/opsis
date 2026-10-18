import { describe, expect, it } from "vitest";
import { FALLBACK_LOOK, NODE_LOOKS } from "@/shared/nodes/looks";
import { NODE_GLYPHS } from "./node-glyphs.generated";

const usedGlyphs = (): Set<string> => {
  const used = new Set<string>([FALLBACK_LOOK.glyph]);
  for (const look of Object.values(NODE_LOOKS)) {
    used.add(look.glyph);
    if (look.morph) used.add(look.morph);
  }
  return used;
};

describe("node glyphs", () => {
  it("holds exactly the glyphs the node looks use (run bun run glyphs:bundle after changing them)", () => {
    const used = usedGlyphs();
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
