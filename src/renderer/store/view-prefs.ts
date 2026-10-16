let project: string | null = null;

const key = (name: string) => `demoflow.view:${project ?? "none"}:${name}`;

export function setViewProject(dir: string | null): void {
  project = dir;
}

export const viewProject = () => project;

export function readPref<T>(name: string, fallback: T, valid: (v: unknown) => boolean): T {
  try {
    const raw = window.localStorage.getItem(key(name));
    if (raw === null) return fallback;
    const value = JSON.parse(raw) as unknown;
    return valid(value) ? (value as T) : fallback;
  } catch {
    return fallback;
  }
}

export function writePref(name: string, value: unknown): void {
  try {
    window.localStorage.setItem(key(name), JSON.stringify(value));
  } catch {
    return;
  }
}

export const layoutKey = (name: string) => `demoflow-${name}:${project ?? "none"}`;
