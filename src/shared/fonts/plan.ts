import type { FontFaceDef } from "./types";

export type FontCandidate = {
  name: string;
  family: string;
  weight: string;
  style: FontFaceDef["style"];
  variable: boolean;
  format: FontFaceDef["format"];
  size: number;
};

export type PlannedFamily<C extends FontCandidate = FontCandidate> = { family: string; faces: C[] };

const STYLE_WORDS =
  /\s+(variable|vf|thin|hairline|extra ?light|ultra ?light|light|regular|book|normal|medium|semi ?bold|demi ?bold|bold|extra ?bold|ultra ?bold|black|heavy|italic|oblique)$/i;

export function familyNameFrom(preferred: string | null | undefined, legacy: string): string {
  let name = (preferred || legacy).trim();
  if (preferred) return name.replace(/\s+(variable|vf)$/i, "");
  for (let prev = ""; prev !== name;) {
    prev = name;
    name = name.replace(STYLE_WORDS, "");
  }
  return name || legacy;
}

const FORMAT_RANK: Record<FontFaceDef["format"], number> = {
  woff2: 0,
  woff: 1,
  truetype: 2,
  opentype: 3,
};

export function planFontFiles<C extends FontCandidate>(candidates: C[]): Array<PlannedFamily<C>> {
  const byFamily = new Map<string, C[]>();
  for (const c of candidates) {
    const list = byFamily.get(c.family) ?? [];
    list.push(c);
    byFamily.set(c.family, list);
  }
  const out: Array<PlannedFamily<C>> = [];
  for (const [family, list] of byFamily) {
    const faces: C[] = [];
    for (const style of ["normal", "italic"] as const) {
      const ofStyle = list.filter((c) => c.style === style);
      const variable = ofStyle.filter((c) => c.variable);
      const pool = variable.length ? variable : ofStyle;
      const byWeight = new Map<string, C>();
      for (const c of pool) {
        const best = byWeight.get(c.weight);
        if (!best || FORMAT_RANK[c.format] < FORMAT_RANK[best.format]) byWeight.set(c.weight, c);
      }
      faces.push(
        ...[...byWeight.values()].toSorted(
          (a, b) => Number.parseInt(a.weight, 10) - Number.parseInt(b.weight, 10),
        ),
      );
    }
    if (faces.length) out.push({ family, faces });
  }
  return out.toSorted((a, b) => a.family.localeCompare(b.family));
}
