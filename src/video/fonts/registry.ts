import { continueRender, delayRender } from "remotion";
import { findFamily, ROLE_ALIASES, roleFamilyName } from "../../shared/fonts";
import {
  DEFAULT_PROJECT_FONTS,
  type FontFamilyDef,
  type ProjectFonts,
} from "../../shared/fonts/types";
import { resolveAssetRef } from "../assets";

type Resolve = (ref: string) => string;
type FontContext = { fonts: ProjectFonts; resolve: Resolve };
type Entry = { signature: string; faces: FontFace[]; ready: Promise<void> };

const registered = new Map<string, Entry>();
let current: FontContext = {
  fonts: DEFAULT_PROJECT_FONTS,
  resolve: (ref) => resolveAssetRef(ref, {}),
};

const canRegister = () => typeof document !== "undefined" && typeof FontFace !== "undefined";

const urlsOf = (def: FontFamilyDef, ctx: FontContext) => def.faces.map((f) => ctx.resolve(f.file));

const signatureOf = (def: FontFamilyDef, urls: string[]) => `${def.id}|${urls.join("|")}`;

const settled = (face: FontFace) => face.status === "loaded" || face.status === "error";

function register(cssName: string, def: FontFamilyDef, ctx: FontContext): Promise<void> {
  if (!canRegister()) return Promise.resolve();
  const urls = urlsOf(def, ctx);
  const signature = signatureOf(def, urls);
  const existing = registered.get(cssName);
  if (existing?.signature === signature) return existing.ready;
  for (const face of existing?.faces ?? []) document.fonts.delete(face);
  const faces = def.faces.map(
    (f, i) =>
      new FontFace(cssName, `url("${urls[i]}") format("${f.format}")`, {
        weight: f.weight,
        style: f.style,
        display: "block",
      }),
  );
  for (const face of faces) document.fonts.add(face);
  const ready = Promise.allSettled(faces.map((f) => f.load())).then(() => undefined);
  registered.set(cssName, { signature, faces, ready });
  return ready;
}

function targetFor(name: string, ctx: FontContext): FontFamilyDef | undefined {
  const role = ROLE_ALIASES[name];
  if (role)
    return findFamily(roleFamilyName(role, ctx.fonts), ctx.fonts) ?? findFamily(name, ctx.fonts);
  return findFamily(name, ctx.fonts);
}

function ensureFont(name: string, ctx: FontContext): Promise<void> {
  const def = targetFor(name, ctx);
  return def ? register(name, def, ctx) : Promise.resolve();
}

function isLoaded(name: string, ctx: FontContext): boolean {
  const def = targetFor(name, ctx);
  const entry = registered.get(name);
  if (!def || !entry?.faces.every(settled)) return false;
  return entry.signature === signatureOf(def, urlsOf(def, ctx));
}

export function ensureFontNow(name: string): void {
  const ctx = current;
  if (!canRegister() || isLoaded(name, ctx)) return;
  const handle = delayRender(`Loading font ${name}`);
  void ensureFont(name, ctx).finally(() => continueRender(handle));
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
