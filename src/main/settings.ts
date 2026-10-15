import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { app } from "electron";
import { appSettingsSchema, DEFAULT_SETTINGS, type AppSettings } from "../shared/settings";

const file = () => path.join(app.getPath("userData"), "settings.json");

let cached: AppSettings | null = null;

export async function readSettings(): Promise<AppSettings> {
  if (cached) return cached;
  try {
    cached = appSettingsSchema.parse(JSON.parse(await readFile(file(), "utf8")));
  } catch {
    cached = DEFAULT_SETTINGS;
  }
  return cached;
}

export async function writeSettings(next: AppSettings): Promise<AppSettings> {
  cached = appSettingsSchema.parse(next);
  await mkdir(path.dirname(file()), { recursive: true });
  await writeFile(file(), `${JSON.stringify(cached, null, 2)}\n`);
  return cached;
}
