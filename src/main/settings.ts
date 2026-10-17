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
import { atomicWrite, readJsonKeepingBroken } from "../engine/json";
import { errorMessage } from "../shared/errors";

const file = () => path.join(app.getPath("userData"), "settings.json");

let cached: AppSettings | null = null;
const listeners = new Set<(settings: AppSettings, previous: AppSettings) => void>();

export function onSettingsChange(
  listener: (settings: AppSettings, previous: AppSettings) => void,
): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

async function storedSettings(): Promise<AppSettings | null> {
  try {
    return await readJsonKeepingBroken(file(), appSettingsSchema);
  } catch (error) {
    process.stderr.write(`[settings] ${errorMessage(error)}\n`);
    return null;
  }
}

export async function readSettings(): Promise<AppSettings> {
  cached ??= (await storedSettings()) ?? DEFAULT_SETTINGS;
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
