import { logError } from "../engine/log";
import { listenerSet } from "../shared/listener-set";
import { keysOf } from "../shared/nodes/params";
import {
  appSettingsSchema,
  DEFAULT_SETTINGS,
  mergeSettings,
  type AppSettings,
  type SettingsPatch,
} from "../shared/settings";
import { db } from "./db";
import { settings as settingsTable } from "./db/schema";

const changes = listenerSet<[settings: AppSettings, previous: AppSettings]>();

export const onSettingsChange = changes.add;

export function readSettings(): AppSettings {
  const rows = db().select().from(settingsTable).all();
  const parsed = appSettingsSchema.safeParse(
    Object.fromEntries(rows.map((row) => [row.section, row.value])),
  );
  if (parsed.success) return parsed.data;
  logError("settings", parsed.error);
  return DEFAULT_SETTINGS;
}

const forgetLastDirOnNewDefault = (next: AppSettings, previous: AppSettings): AppSettings =>
  next.projects.defaultDir !== previous.projects.defaultDir && next.projects.lastDir
    ? { ...next, projects: { ...next.projects, lastDir: "" } }
    : next;

export function patchSettings(patch: SettingsPatch): AppSettings {
  const previous = readSettings();
  const next = forgetLastDirOnNewDefault(mergeSettings(previous, patch), previous);
  db().transaction((tx) => {
    for (const section of keysOf(next))
      tx.insert(settingsTable)
        .values({ section, value: next[section] })
        .onConflictDoUpdate({ target: settingsTable.section, set: { value: next[section] } })
        .run();
  });
  changes.emit(next, previous);
  return next;
}
