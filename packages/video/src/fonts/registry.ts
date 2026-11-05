import { assetUrl } from "#video/assets.tsx";
import { registerFaces, registeredNames } from "#video/common/font-faces.ts";
import type { FilmFonts, FontFamilyDef } from "#video/generated/spec.ts";

export const NO_FONTS: FilmFonts = { families: {}, preload: [] };

function familyNamed(name: string, fonts: FilmFonts): FontFamilyDef | undefined {
  return fonts.families[name.toLowerCase()];
}

function ensureFont(name: string, fonts: FilmFonts): Promise<void> {
  const def = familyNamed(name, fonts);
  return def ? registerFaces(name, def, assetUrl) : Promise.resolve();
}

export function applyFilmFonts(fonts: FilmFonts): Promise<void> {
  const names = new Set([...fonts.preload, ...registeredNames()]);
  return Promise.all([...names].map((name) => ensureFont(name, fonts))).then(() => undefined);
}
