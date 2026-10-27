import { describe, expect, it } from "vitest";
import { GLYPH_SETS, geometryOnly, loadGlyph } from "./glyph-sets";

const ALL_NOTICES = Object.values(GLYPH_SETS)
  .map(({ notice }) => `## ${notice}`)
  .join("\n");
const GEOMETRY = /^(d|cx|cy|r|rx|ry|x|y|width|height|x1|y1|x2|y2|points)$/;

describe("glyph sets", () => {
  it.each([
    "lucide:camera",
    "tabler:camera",
    "lucide-lab:apple-core",
    "hugeicons:camera-01",
    "iconoir:camera",
    "mynaui:camera",
  ])("reads %s as shapes with nothing but their geometry", async (glyph) => {
    const node = await loadGlyph(glyph, ALL_NOTICES);
    expect(node.length).toBeGreaterThan(0);
    for (const [tag, attrs] of node) {
      expect(["path", "line", "circle", "rect", "polyline", "polygon", "ellipse"]).toContain(tag);
      for (const name of Object.keys(attrs)) expect(name).toMatch(GEOMETRY);
    }
  });

  it("refuses typos, aliases and sets it doesn't know", async () => {
    await expect(loadGlyph("lucide:camra", ALL_NOTICES)).rejects.toThrow("an alias or a typo");
    await expect(loadGlyph("hugeicons:agreement", ALL_NOTICES)).rejects.toThrow(
      "an alias or a typo",
    );
    await expect(loadGlyph("iconoir:camra", ALL_NOTICES)).rejects.toThrow("not an outline icon");
    await expect(loadGlyph("phosphor:camera", ALL_NOTICES)).rejects.toThrow("isn't from one of");
  });

  it("asks for a set's licence notice before using its glyphs", async () => {
    await expect(loadGlyph("hugeicons:camera-01", "## Lucide icons")).rejects.toThrow(
      "needs the Hugeicons licence in THIRD_PARTY_NOTICES.md first",
    );
  });

  it("refuses filled parts, groups and other attributes", () => {
    expect(() => geometryOnly("x:a", [["path", { d: "M0 0", fill: "currentColor" }]])).toThrow(
      "has a filled part",
    );
    expect(() => geometryOnly("x:a", [["g", {}]])).toThrow("draws a <g>");
    expect(() => geometryOnly("x:a", [["path", { d: "M0 0", opacity: "0.5" }]])).toThrow(
      "sets opacity",
    );
    expect(
      geometryOnly("x:a", [["path", { d: "M0 0", "stroke-width": "1.5", stroke: "currentColor" }]]),
    ).toEqual([["path", { d: "M0 0" }]]);
  });
});
