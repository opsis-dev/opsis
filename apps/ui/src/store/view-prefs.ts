import type { ViewPrefChange, ViewPrefs } from "@opsis/core/ipc";

let project: string | null = null;
let prefs: ViewPrefs = {};

export function setViewProject(dir: string | null, stored: ViewPrefs): void {
  project = dir;
  prefs = { ...stored };
}

export const viewPrefs = (): ViewPrefs => prefs;

export function readPref<T>(name: string, fallback: T, valid: (v: unknown) => v is T): T {
  const value = prefs[name];
  return Object.hasOwn(prefs, name) && valid(value) ? value : fallback;
}

export function writePref(name: string, value: unknown): void {
  prefs = { ...prefs, [name]: value };
  if (project)
    window.opsis
      .setViewPref(project, name, value)
      .catch((error: unknown) => console.error("A view setting couldn't be saved.", error));
}

function applyViewPref({ dir, key, value }: ViewPrefChange): void {
  if (dir === project) prefs = { ...prefs, [key]: value };
}

export function watchViewPrefs(): void {
  window.opsis.onViewPrefChanged(applyViewPref);
}
