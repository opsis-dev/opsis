import { BUNDLED_FONTS } from "./bundled.generated";
import type { FontFamilyDef, ProjectFonts } from "./types";

export * from "./types";
export { BUNDLED_FONTS };

export type FontRole = "main" | "heading" | "code";

export const ROLE_FAMILIES: Record<FontRole, string> = {
  main: "df-main",
  heading: "df-heading",
  code: "df-code",
};

export const ROLE_ALIASES: Record<string, FontRole> = {
  "df-main": "main",
  "df-heading": "heading",
  "df-code": "code",
  Inter: "main",
  "JetBrains Mono": "code",
};

export const FAMILY_ALIASES: Record<string, string> = {
  "Google Sans": "Google Sans Flex",
};

const GENERIC: Record<FontFamilyDef["category"], string> = {
  "sans-serif": "sans-serif",
  serif: "serif",
  monospace: "monospace",
  display: "sans-serif",
  handwriting: "cursive",
};

export function allFamilies(fonts: Pick<ProjectFonts, "families">): FontFamilyDef[] {
  const seen = new Set<string>();
  return [...fonts.families, ...BUNDLED_FONTS].filter((f) => {
    const key = f.family.toLowerCase();
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

export function findFamily(
  name: string,
  fonts: Pick<ProjectFonts, "families">,
): FontFamilyDef | undefined {
  const wanted = (FAMILY_ALIASES[name] ?? name).toLowerCase();
  return allFamilies(fonts).find((f) => f.family.toLowerCase() === wanted);
}

export function roleFamilyName(role: FontRole, fonts: ProjectFonts): string {
  if (role === "heading") return fonts.heading || fonts.main;
  return fonts[role];
}

export function fontStack(name: string, fonts: Pick<ProjectFonts, "families">): string {
  const def = findFamily(name, fonts);
  return `"${name.replaceAll('"', "")}", ${GENERIC[def?.category ?? "sans-serif"]}`;
}

export function roleStack(role: FontRole): string {
  const fallback = role === "code" ? "ui-monospace, monospace" : "system-ui, sans-serif";
  return `"${ROLE_FAMILIES[role]}", ${fallback}`;
}

export function firstFamily(stack: string): string {
  return (stack.split(",")[0] ?? "").trim().replace(/^["']|["']$/g, "");
}

export function fontSlug(family: string): string {
  return (
    family
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "font"
  );
}
