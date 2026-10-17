import { continueRender, delayRender, staticFile } from "remotion";
import { findFamily, ROLE_ALIASES, roleFamilyName } from "../../shared/fonts";
import {
  DEFAULT_PROJECT_FONTS,
  type FontFamilyDef,
  type ProjectFonts,
} from "../../shared/fonts/types";

type Resolve = (ref: string) => string;
type Entry = { signature: string; faces: FontFace[]; ready: Promise<void>; done: boolean };

const FORMATS: Record<FontFamilyDef["faces"][number]["format"], string> = {
  woff2: "woff2",
  woff: "woff",
  truetype: "truetype",
  opentype: "opentype",
};

const staticResolve: Resolve = (ref) =>
  ref.startsWith("app:") ? staticFile(ref.slice("app:".length)) : ref;

const registered = new Map<string, Entry>();
let project: ProjectFonts = DEFAULT_PROJECT_FONTS;
let resolveRef: Resolve = staticResolve;

const canRegister = () => typeof document !== "undefined" && typeof FontFace !== "undefined";

function register(cssName: string, def: FontFamilyDef): Promise<void> {
  if (!canRegister()) return Promise.resolve();
  const urls = def.faces.map((f) => resolveRef(f.file));
  const signature = `${def.id}|${urls.join("|")}`;
  const current = registered.get(cssName);
  if (current?.signature === signature) return current.ready;
  for (const face of current?.faces ?? []) document.fonts.delete(face);
  const faces = def.faces.map(
    (f, i) =>
      new FontFace(cssName, `url("${urls[i]}") format("${FORMATS[f.format]}")`, {
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

export function targetFor(name: string): FontFamilyDef | undefined {
  const role = ROLE_ALIASES[name];
  if (role) return findFamily(roleFamilyName(role, project), project) ?? findFamily(name, project);
  return findFamily(name, project);
}

export function ensureFont(name: string): Promise<void> {
  const def = targetFor(name);
  return def ? register(name, def) : Promise.resolve();
}

function isLoaded(name: string): boolean {
  const def = targetFor(name);
  const entry = registered.get(name);
  if (!def || !entry?.done) return false;
  return entry.signature === `${def.id}|${def.faces.map((f) => resolveRef(f.file)).join("|")}`;
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
  resolveRef = (ref) => (ref.startsWith("app:") ? staticResolve(ref) : resolve(ref));
  const names = new Set([...Object.keys(ROLE_ALIASES), ...registered.keys(), ...used]);
  return Promise.all([...names].map(ensureFont)).then(() => undefined);
}
