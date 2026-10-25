import { findFamily, ROLE_ALIASES, roleFamilyName } from "@opsis/core/fonts";
import {
  DEFAULT_PROJECT_FONTS,
  type FontFamilyDef,
  type ProjectFonts,
} from "@opsis/core/fonts/types";
import { resolveAssetRef } from "#video/assets.tsx";
import { holdRender } from "#video/core/loading.ts";

type Resolve = (ref: string) => string;
type FontContext = { fonts: ProjectFonts; resolve: Resolve };
type Entry = { signature: string; faces: FontFace[]; ready: Promise<void> };

const registered = new Map<string, Entry>();
let current: FontContext = {
  fonts: DEFAULT_PROJECT_FONTS,
  resolve: (ref) => resolveAssetRef(ref, {}),
};

const canRegister = () => typeof document !== "undefined" && typeof FontFace !== "undefined";

const urlsOf = (def: FontFamilyDef, resolve: Resolve) => def.faces.map((f) => resolve(f.file));

const signatureOf = (def: FontFamilyDef, urls: string[]) =>
  [def.id, ...def.faces.map((f, i) => `${urls[i]} ${f.weight} ${f.style}`)].join("|");

const settled = (face: FontFace) => face.status === "loaded" || face.status === "error";

export function unregisterFaces(cssName: string): void {
  for (const face of registered.get(cssName)?.faces ?? []) document.fonts.delete(face);
  registered.delete(cssName);
}

export function registerFaces(
  cssName: string,
  def: FontFamilyDef,
  resolve: Resolve,
  onUnreadable?: () => void,
): Promise<void> {
  if (!canRegister()) return Promise.resolve();
  const urls = urlsOf(def, resolve);
  const signature = signatureOf(def, urls);
  const existing = registered.get(cssName);
  if (existing?.signature === signature) return existing.ready;
  unregisterFaces(cssName);
  const faces = def.faces.map(
    (f, i) =>
      new FontFace(cssName, `url("${urls[i]}") format("${f.format}")`, {
        weight: f.weight,
        style: f.style,
        display: "block",
      }),
  );
  for (const face of faces) document.fonts.add(face);
  const ready = Promise.allSettled(faces.map((f) => f.load())).then((results) => {
    if (results.some((r) => r.status === "rejected")) onUnreadable?.();
  });
  registered.set(cssName, { signature, faces, ready });
  return ready;
}

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
  const entry = registered.get(name);
  if (!def || !entry?.faces.every(settled)) return false;
  return entry.signature === signatureOf(def, urlsOf(def, ctx.resolve));
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
  const names = new Set([...Object.keys(ROLE_ALIASES), ...registered.keys(), ...used]);
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
