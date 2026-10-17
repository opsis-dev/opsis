import { BUNDLED_FONTS } from "./bundled.generated";
import type { FontFamilyDef, ProjectFonts } from "./types";
import { safeName } from "../project-folder";

const FONT_ROLES = ["main", "heading", "code"] as const;

type FontRole = (typeof FONT_ROLES)[number];

const roleFamily = (role: FontRole): string => `df-${role}`;

export const ROLE_ALIASES: Record<string, FontRole> = {
  ...Object.fromEntries(FONT_ROLES.map((role) => [roleFamily(role), role])),
  Inter: "main",
  "JetBrains Mono": "code",
};

const FAMILY_ALIASES: Partial<Record<string, string>> = {
  "Google Sans": "Google Sans Flex",
};

const GENERIC: Record<FontFamilyDef["category"], string> = {
  "sans-serif": "sans-serif",
  serif: "serif",
  monospace: "monospace",
  display: "sans-serif",
  handwriting: "cursive",
};

function allFamilies(fonts: Pick<ProjectFonts, "families">): FontFamilyDef[] {
  const seen = new Set<string>();
  return [...fonts.families, ...BUNDLED_FONTS].filter((f) => {
    const key = familyKey(f.family);
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

export function findFamily(
  name: string,
  fonts: Pick<ProjectFonts, "families">,
): FontFamilyDef | undefined {
  const wanted = FAMILY_ALIASES[name] ?? name;
  return allFamilies(fonts).find((f) => sameFamily(f.family, wanted));
}

export function roleFamilyName(role: FontRole, fonts: ProjectFonts): string {
  if (role === "heading") return fonts.heading || fonts.main;
  return fonts[role];
}

export function fontStack(
  name: string,
  fonts: Pick<ProjectFonts, "families"> = { families: [] },
): string {
  const def = findFamily(name, fonts);
  return `"${name.replaceAll('"', "")}", ${GENERIC[def?.category ?? "sans-serif"]}`;
}

export function roleStack(role: FontRole): string {
  const fallback = role === "code" ? "ui-monospace, monospace" : "system-ui, sans-serif";
  return `"${roleFamily(role)}", ${fallback}`;
}

export function firstFamily(stack: string): string {
  return stack
    .split(",")[0]
    .trim()
    .replace(/^["']|["']$/g, "");
}

export const fontSlug = (family: string): string =>
  safeName(family, { fallback: "font", alphabet: "alnum" });

export const familyKey = (family: string): string => family.toLowerCase();

export const sameFamily = (a: string, b: string): boolean => familyKey(a) === familyKey(b);
