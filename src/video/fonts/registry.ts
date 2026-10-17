import { continueRender, delayRender } from "remotion";
import { findFamily, ROLE_ALIASES, roleFamilyName } from "../../shared/fonts";
import {
  DEFAULT_PROJECT_FONTS,
  type FontFamilyDef,
  type ProjectFonts,
} from "../../shared/fonts/types";
import { resolveAssetRef } from "../assets";

type Resolve = (ref: string) => string;
type Entry = { signature: string; faces: FontFace[]; ready: Promise<void>; done: boolean };

const registered = new Map<string, Entry>();
let project: ProjectFonts = DEFAULT_PROJECT_FONTS;
let resolveRef: Resolve = (ref) => resolveAssetRef(ref, {});

const canRegister = () => typeof document !== "undefined" && typeof FontFace !== "undefined";

const urlsOf = (def: FontFamilyDef) => def.faces.map((f) => resolveRef(f.file));

const signatureOf = (def: FontFamilyDef, urls: string[]) => `${def.id}|${urls.join("|")}`;

function register(cssName: string, def: FontFamilyDef): Promise<void> {
  if (!canRegister()) return Promise.resolve();
  const urls = urlsOf(def);
  const signature = signatureOf(def, urls);
  const current = registered.get(cssName);
  if (current?.signature === signature) return current.ready;
  for (const face of current?.faces ?? []) document.fonts.delete(face);
  const faces = def.faces.map(
    (f, i) =>
      new FontFace(cssName, `url("${urls[i]}") format("${f.format}")`, {
        weight: f.weight,
        style: f.style,
        display: "block",
      }),
  );
  for (const face of faces) document.fonts.add(face);
  const entry: Entry = { signature, faces, ready: Promise.resolve(), done: false };
  entry.ready = Promise.allSettled(faces.map((f) => f.load())).then(() => {
    entry.done = true;
  });
  registered.set(cssName, entry);
  return entry.ready;
}

function targetFor(name: string): FontFamilyDef | undefined {
  const role = ROLE_ALIASES[name];
  if (role) return findFamily(roleFamilyName(role, project), project) ?? findFamily(name, project);
  return findFamily(name, project);
}

function ensureFont(name: string): Promise<void> {
  const def = targetFor(name);
  return def ? register(name, def) : Promise.resolve();
}

function isLoaded(name: string): boolean {
  const def = targetFor(name);
  const entry = registered.get(name);
  if (!def || !entry?.done) return false;
  return entry.signature === signatureOf(def, urlsOf(def));
}

export function ensureFontNow(name: string): void {
  if (!canRegister() || isLoaded(name)) return;
  const handle = delayRender(`Loading font ${name}`);
  void ensureFont(name).finally(() => continueRender(handle));
}

export function applyProjectFonts(
  fonts: ProjectFonts,
  resolve: Resolve,
  used: string[],
): Promise<void> {
  project = fonts;
  resolveRef = resolve;
  const names = new Set([...Object.keys(ROLE_ALIASES), ...registered.keys(), ...used]);
  return Promise.all([...names].map(ensureFont)).then(() => undefined);
}
