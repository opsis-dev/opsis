import { tryParseJson } from "@/shared/guards";
import { readStored, writeStored } from "@/renderer/lib/local-storage";

let project: string | null = null;

const key = (name: string) => `demoflow.view:${project ?? "none"}:${name}`;

export function setViewProject(dir: string | null): void {
  project = dir;
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
