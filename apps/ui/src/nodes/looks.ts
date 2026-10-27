import type { NodeLook } from "@opsis/video/generated/spec";

export const FALLBACK_LOOK: NodeLook = { glyph: "lucide:box" };

export function usedGlyphs(looks: readonly NodeLook[]): string[] {
  const used = new Set<string>([FALLBACK_LOOK.glyph]);
  for (const look of looks) {
    used.add(look.glyph);
    if (look.morph) used.add(look.morph);
  }
  return [...used].sort();
}
