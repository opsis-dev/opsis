import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { app, shell } from "electron";
import type { LicenceEntry } from "../shared/ipc";
import { environment } from "./workers";

const resourceDir = () => (app.isPackaged ? process.resourcesPath : app.getAppPath());
const runtimeDir = () => path.dirname(process.execPath);

async function text(file: string): Promise<string | undefined> {
  try {
    return await readFile(file, "utf8");
  } catch {
    return undefined;
  }
}

function sources(): Array<Omit<LicenceEntry, "text"> & { read?: string }> {
  const engine = environment().engine.dir;
  return [
    {
      id: "demoflow",
      title: "Demoflow",
      summary: "Apache License 2.0",
      read: path.join(resourceDir(), "LICENSE"),
    },
    {
      id: "third-party",
      title: "Included source code",
      summary: "Notices for the components and code Demoflow includes",
      read: path.join(resourceDir(), "THIRD_PARTY_NOTICES.md"),
    },
    {
      id: "electron",
      title: "Electron",
      summary: "MIT License. The app window runs on Electron.",
      read: path.join(runtimeDir(), "LICENSE"),
    },
    {
      id: "app-chromium",
      title: "Chromium in the app window",
      summary: "Credits and licences of the browser the app window is built on",
      file: path.join(runtimeDir(), "LICENSES.chromium.html"),
    },
    {
      id: "engine-chromium",
      title: "Recording browser",
      summary: "Credits and licences of the Chromium build that records takes and exports videos",
      file: path.join(engine, "LICENSES.chromium.html"),
    },
    {
      id: "ffmpeg",
      title: "FFmpeg",
      summary: "GNU General Public License v3. Separate programs that encode and mix the video.",
      read: path.join(engine, "ffmpeg", "LICENSE.txt"),
    },
  ];
}

let allowed = new Set<string>();

export async function listLicences(): Promise<LicenceEntry[]> {
  const out: LicenceEntry[] = [];
  for (const { read, ...entry } of sources()) {
    if (read) {
      const body = await text(read);
      if (body) out.push({ ...entry, text: body });
    } else if (entry.file && existsSync(entry.file)) out.push(entry);
  }
  allowed = new Set(out.flatMap((e) => (e.file ? [e.file] : [])));
  return out;
}

export async function openLicenceFile(file: string): Promise<void> {
  if (!allowed.has(file)) throw new Error("That file isn't one of the licence files.");
  const error = await shell.openPath(file);
  if (error) throw new Error(error);
}
