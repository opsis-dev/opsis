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

function stripStyleWords(name: string): string {
  const stripped = name.replace(STYLE_WORDS, "");
  return stripped === name ? name : stripStyleWords(stripped);
}

export function familyNameFrom(preferred: string | null | undefined, legacy: string): string {
  if (preferred) return preferred.trim().replace(/\s+(variable|vf)$/i, "");
  return stripStyleWords(legacy.trim()) || legacy;
}

const FORMAT_RANK: Record<FontFaceDef["format"], number> = {
  woff2: 0,
  woff: 1,
  truetype: 2,
  opentype: 3,
};

function groupByFamily<C extends FontCandidate>(candidates: C[]): Map<string, C[]> {
  const byFamily = new Map<string, C[]>();
  for (const c of candidates) {
    const list = byFamily.get(c.family);
    if (list) list.push(c);
    else byFamily.set(c.family, [c]);
  }
  return byFamily;
}

function pickFaces<C extends FontCandidate>(list: C[], style: FontFaceDef["style"]): C[] {
  const ofStyle = list.filter((c) => c.style === style);
  const variable = ofStyle.filter((c) => c.variable);
  const byWeight = new Map<string, C>();
  for (const c of variable.length ? variable : ofStyle) {
    const best = byWeight.get(c.weight);
    if (!best || FORMAT_RANK[c.format] < FORMAT_RANK[best.format]) byWeight.set(c.weight, c);
  }
  return [...byWeight.values()].toSorted(
    (a, b) => Number.parseInt(a.weight, 10) - Number.parseInt(b.weight, 10),
  );
}

export function planFontFiles<C extends FontCandidate>(candidates: C[]): Array<PlannedFamily<C>> {
  return [...groupByFamily(candidates)]
    .map(([family, list]) => ({
      family,
      faces: [...pickFaces(list, "normal"), ...pickFaces(list, "italic")],
    }))
    .filter((f) => f.faces.length)
    .toSorted((a, b) => a.family.localeCompare(b.family));
}
