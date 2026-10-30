import type { FontFamilyDef } from "#video/generated/spec.ts";

export type Resolve = (ref: string) => string;

type Entry = { signature: string; faces: FontFace[]; ready: Promise<void> };

const registered = new Map<string, Entry>();

export function canRegister() {
  return typeof document !== "undefined" && typeof FontFace !== "undefined";
}

export function urlsOf(def: FontFamilyDef, resolve: Resolve) {
  return def.faces.map((f) => resolve(f.file));
}

export function signatureOf(def: FontFamilyDef, urls: string[]) {
  return [def.id, ...def.faces.map((f, i) => `${urls[i]} ${f.weight} ${f.style}`)].join("|");
}

function settled(face: FontFace) {
  return face.status === "loaded" || face.status === "error";
}

export function registeredNames(): string[] {
  return [...registered.keys()];
}

export function loadedSignature(cssName: string): string | null {
  const entry = registered.get(cssName);
  return entry?.faces.every(settled) ? entry.signature : null;
}

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
