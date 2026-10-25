import { BUNDLED_FONTS } from "#core/fonts/bundled.generated";
import type { FontFamilyDef, ProjectFonts } from "#core/fonts/types";
import { nonBlankString } from "#core/guards";
import { safeName } from "#core/safe-name";

const FONT_ROLES = ["main", "heading", "code"] as const;

export type FontRole = (typeof FONT_ROLES)[number];

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

export function findFamily(
  name: string,
  fonts: Pick<ProjectFonts, "families">,
): FontFamilyDef | undefined {
  const wanted = FAMILY_ALIASES[name] ?? name;
  return [...fonts.families, ...BUNDLED_FONTS].find((f) => sameFamily(f.family, wanted));
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

export function fontProp(value: unknown): string | undefined {
  const name = nonBlankString(value);
  return name ? fontStack(name) : undefined;
}

function roleStack(role: FontRole): string {
  const fallback = role === "code" ? "ui-monospace, monospace" : "system-ui, sans-serif";
  return `"${roleFamily(role)}", ${fallback}`;
}

export const fontOr = (family: string | undefined, role: FontRole): string =>
  family || roleStack(role);

export const MAIN_FONT = roleStack("main");

export const CODE_FONT = roleStack("code");

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

type Named = ReadonlyArray<Pick<FontFamilyDef, "family">>;

export const familyKeysOf = (fonts: Named): ReadonlySet<string> =>
  new Set(fonts.map((f) => familyKey(f.family)));

export const familyKeysWithBundled = (fonts: Named): ReadonlySet<string> =>
  familyKeysOf([...fonts, ...BUNDLED_FONTS]);
