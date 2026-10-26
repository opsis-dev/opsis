import { assetUrl } from "#video/assets.tsx";
import {
  canRegister,
  loadedSignature,
  registerFaces,
  registeredNames,
  signatureOf,
  urlsOf,
} from "#video/common/font-faces.ts";
import { holdRender } from "#video/core/loading.ts";
import type { FilmFonts, FontFamilyDef } from "#video/generated/spec.ts";

export const NO_FONTS: FilmFonts = { families: {}, preload: [] };

let current: FilmFonts = NO_FONTS;

const familyNamed = (name: string, fonts: FilmFonts): FontFamilyDef | undefined =>
  fonts.families[name.toLowerCase()];

function ensureFont(name: string, fonts: FilmFonts): Promise<void> {
  const def = familyNamed(name, fonts);
  return def ? registerFaces(name, def, assetUrl) : Promise.resolve();
}

function isLoaded(name: string, fonts: FilmFonts): boolean {
  const def = familyNamed(name, fonts);
  return !!def && loadedSignature(name) === signatureOf(def, urlsOf(def, assetUrl));
}

export function ensureFontNow(name: string): void {
  const fonts = current;
  if (!canRegister() || isLoaded(name, fonts)) return;
  const release = holdRender(`Loading font ${name}`);
  void ensureFont(name, fonts).finally(release);
}

export function applyFilmFonts(fonts: FilmFonts): Promise<void> {
  current = fonts;
  const names = new Set([...fonts.preload, ...registeredNames()]);
  return Promise.all([...names].map((name) => ensureFont(name, fonts))).then(() => undefined);
}
