import { mkdir } from "node:fs/promises";
import path from "node:path";
import { app } from "electron";
import { readJsonKeepingBroken, writeJson } from "../engine/json";
import { logError } from "../engine/log";
import {
  appSettingsSchema,
  DEFAULT_SETTINGS,
  mergeSettings,
  type AppSettings,
  type SettingsPatch,
} from "../shared/settings";
import { oneAtATime } from "../shared/timing";

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
    logError("settings", error);
    return null;
  }
}

export async function readSettings(): Promise<AppSettings> {
  cached ??= (await storedSettings()) ?? DEFAULT_SETTINGS;
  return cached;
}

const forgetLastDirOnNewDefault = (next: AppSettings, previous: AppSettings): AppSettings =>
  next.projects.defaultDir !== previous.projects.defaultDir && next.projects.lastDir
    ? { ...next, projects: { ...next.projects, lastDir: "" } }
    : next;

async function writeSettings(patch: SettingsPatch): Promise<AppSettings> {
  const previous = await readSettings();
  const next = appSettingsSchema.parse(
    forgetLastDirOnNewDefault(mergeSettings(previous, patch), previous),
  );
  await mkdir(path.dirname(file()), { recursive: true });
  await writeJson(file(), next);
  cached = next;
  for (const listener of listeners) listener(next, previous);
  return next;
}

const oneWriteAtATime = oneAtATime();

export const patchSettings = (patch: SettingsPatch): Promise<AppSettings> =>
  oneWriteAtATime(() => writeSettings(patch));
