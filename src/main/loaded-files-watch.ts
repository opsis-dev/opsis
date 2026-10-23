import { existsSync, watch, type FSWatcher } from "node:fs";
import path from "node:path";
import { fileStamp } from "../engine/file-stamps";
import { logError } from "../engine/log";
import { fileVersionKey, staysInProject } from "../shared/asset-ref";
import type { ProjectFilesChange } from "../shared/ipc";
import { PROJECT_DIRS } from "../shared/project-dirs";
import { debounce } from "../shared/timing";

export const MISSING_FILE = "missing";

const NOTIFY_DEBOUNCE_MS = 150;

export type LoadedFilesWatch = { follow: (refs: readonly string[]) => void; close: () => void };

const folderOf = (key: string): string => key.split("/").slice(0, -1).join("/");

const isAppMade = (key: string): boolean =>
  key === PROJECT_DIRS.meta || key.startsWith(`${PROJECT_DIRS.meta}/`);

function watchedKeys(refs: readonly string[]): Set<string> {
  const keys = refs
    .filter(staysInProject)
    .map(fileVersionKey)
    .filter((key) => key !== null)
    .filter((key) => !isAppMade(key));
  return new Set(keys);
}

export function watchLoadedFiles(
  dir: string,
  notify: (change: ProjectFilesChange) => void,
): LoadedFilesWatch {
  const watchers = new Map<string, FSWatcher>();
  const pending = new Set<string>();
  let loaded = new Set<string>();

  const flush = debounce(() => {
    const files = Object.fromEntries(
      [...pending].map((key) => [key, fileStamp(path.join(dir, key)) ?? MISSING_FILE]),
    );
    pending.clear();
    notify({ dir, files });
  }, NOTIFY_DEBOUNCE_MS);

  const changed = (folder: string, name: string | null) => {
    const key = name && fileVersionKey(folder ? `${folder}/${name}` : name);
    if (!key || !loaded.has(key)) return;
    pending.add(key);
    flush();
  };

  const open = (folder: string) => {
    const full = path.join(dir, folder);
    if (!existsSync(full)) return;
    try {
      const watcher = watch(full, (_event, name) => changed(folder, name));
      watcher.on("error", (error) => logError("projects", error, `watching ${full}`));
      watchers.set(folder, watcher);
    } catch (error) {
      logError("projects", error, `watching ${full}`);
    }
  };

  return {
    follow: (refs) => {
      loaded = watchedKeys(refs);
      const folders = new Set([...loaded].map(folderOf));
      for (const [folder, watcher] of watchers) {
        if (folders.has(folder)) continue;
        watcher.close();
        watchers.delete(folder);
      }
      for (const folder of folders) if (!watchers.has(folder)) open(folder);
    },
    close: () => {
      flush.cancel();
      for (const watcher of watchers.values()) watcher.close();
      watchers.clear();
    },
  };
}
