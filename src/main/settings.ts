import { mkdir } from "node:fs/promises";
import path from "node:path";
import { app } from "electron";
import {
  appSettingsSchema,
  DEFAULT_SETTINGS,
  mergeSettings,
  type AppSettings,
  type SettingsPatch,
} from "../shared/settings";
import { atomicWrite, readJson } from "../engine/json";

const file = () => path.join(app.getPath("userData"), "settings.json");

let cached: AppSettings | null = null;
const listeners = new Set<(settings: AppSettings, previous: AppSettings) => void>();

export function onSettingsChange(
  listener: (settings: AppSettings, previous: AppSettings) => void,
): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export async function readSettings(): Promise<AppSettings> {
  if (cached) return cached;
  cached = appSettingsSchema.safeParse(await readJson(file())).data ?? DEFAULT_SETTINGS;
  return cached;
}

export async function writeSettings(next: AppSettings): Promise<AppSettings> {
  const previous = await readSettings();
  cached = appSettingsSchema.parse(next);
  await mkdir(path.dirname(file()), { recursive: true });
  await atomicWrite(file(), `${JSON.stringify(cached, null, 2)}\n`);
  for (const listener of listeners) listener(cached, previous);
  return cached;
}

export async function patchSettings(patch: SettingsPatch): Promise<AppSettings> {
  return writeSettings(mergeSettings(await readSettings(), patch));
}
