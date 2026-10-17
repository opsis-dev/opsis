import { tryParseJson } from "@/shared/guards";

let project: string | null = null;

const key = (name: string) => `demoflow.view:${project ?? "none"}:${name}`;

export function setViewProject(dir: string | null): void {
  project = dir;
}

export function readStored(storageKey: string): string | null {
  try {
    return window.localStorage.getItem(storageKey);
  } catch {
    return null;
  }
}

export function writeStored(storageKey: string, value: string): void {
  try {
    window.localStorage.setItem(storageKey, value);
  } catch {
    return;
  }
}

export function readPref<T>(name: string, fallback: T, valid: (v: unknown) => v is T): T {
  const raw = readStored(key(name));
  if (raw === null) return fallback;
  const parsed = tryParseJson(raw);
  return parsed.ok && valid(parsed.value) ? parsed.value : fallback;
}

export function writePref(name: string, value: unknown): void {
  writeStored(key(name), JSON.stringify(value));
}
