import { resolveAssetRef } from "#video/assets.tsx";
import {
  canRegister,
  loadedSignature,
  registerFaces,
  registeredNames,
  signatureOf,
  urlsOf,
  type Resolve,
} from "#video/common/font-faces.ts";
import {
  findFamily,
  ROLE_ALIASES,
  roleFamilyName,
  DEFAULT_PROJECT_FONTS,
} from "#video/common/fonts.ts";
import { holdRender } from "#video/core/loading.ts";
import type { FontFamilyDef, ProjectFonts } from "#video/generated/spec.ts";

type FontContext = { fonts: ProjectFonts; resolve: Resolve };

let current: FontContext = {
  fonts: DEFAULT_PROJECT_FONTS,
  resolve: (ref) => resolveAssetRef(ref, {}),
};

function targetFor(name: string, ctx: FontContext): FontFamilyDef | undefined {
  const role = ROLE_ALIASES[name];
  const fromRole = role ? findFamily(roleFamilyName(role, ctx.fonts), ctx.fonts) : undefined;
  return fromRole ?? findFamily(name, ctx.fonts);
}

function ensureFont(name: string, ctx: FontContext): Promise<void> {
  const def = targetFor(name, ctx);
  return def ? registerFaces(name, def, ctx.resolve) : Promise.resolve();
}

function isLoaded(name: string, ctx: FontContext): boolean {
  const def = targetFor(name, ctx);
  const signature = loadedSignature(name);
  return !!def && signature === signatureOf(def, urlsOf(def, ctx.resolve));
}

export function ensureFontNow(name: string): void {
  const ctx = current;
  if (!canRegister() || isLoaded(name, ctx)) return;
  const release = holdRender(`Loading font ${name}`);
  void ensureFont(name, ctx).finally(release);
}

export function applyProjectFonts(
  fonts: ProjectFonts,
  resolve: Resolve,
  used: string[],
): Promise<void> {
  const ctx = { fonts, resolve };
  current = ctx;
  const names = new Set([...Object.keys(ROLE_ALIASES), ...registeredNames(), ...used]);
  return Promise.all([...names].map((name) => ensureFont(name, ctx))).then(() => undefined);
}

export type FontFile = { url: string; italic: boolean };

const weightRange = (weight: string): [number, number] => {
  const [low = 400, high = low] = weight.split(/\s+/).map(Number).filter(Number.isFinite);
  return [low, high];
};

const weightDistance = (weight: string, wanted: number): number => {
  const [low, high] = weightRange(weight);
  return wanted < low ? low - wanted : wanted > high ? wanted - high : 0;
};

export function fontFileFor(name: string, weight: number, italic: boolean): FontFile | null {
  const def = targetFor(name, current);
  if (!def?.faces.length) return null;
  const styled = def.faces.filter((f) => (f.style === "italic") === italic);
  const faces = styled.length ? styled : def.faces;
  const face = faces.toSorted(
    (a, b) => weightDistance(a.weight, weight) - weightDistance(b.weight, weight),
  )[0];
  return face ? { url: current.resolve(face.file), italic: face.style === "italic" } : null;
}
