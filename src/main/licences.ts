import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { app, shell } from "electron";
import { CHROMIUM_CREDITS_FILE } from "../engine/paths";
import type { LicenceEntry } from "../shared/ipc";
import { environment } from "./engine-setup";

const resourceDir = () => (app.isPackaged ? process.resourcesPath : app.getAppPath());
const runtimeDir = () => path.dirname(process.execPath);

type LicenceSource = Omit<LicenceEntry, "text" | "file"> & { kind: "text" | "file"; path: string };

function sources(): LicenceSource[] {
  const engine = environment().engine.dir;
  return [
    {
      id: "demoflow",
      title: "Demoflow",
      summary: "Apache License 2.0",
      kind: "text",
      path: path.join(resourceDir(), "LICENSE"),
    },
    {
      id: "third-party",
      title: "Included source code",
      summary: "Notices for the components and code Demoflow includes",
      kind: "text",
      path: path.join(resourceDir(), "THIRD_PARTY_NOTICES.md"),
    },
    {
      id: "electron",
      title: "Electron",
      summary: "MIT License. The app window runs on Electron.",
      kind: "text",
      path: path.join(runtimeDir(), "LICENSE"),
    },
    {
      id: "app-chromium",
      title: "Chromium in the app window",
      summary: "Credits and licences of the browser the app window is built on",
      kind: "file",
      path: path.join(runtimeDir(), CHROMIUM_CREDITS_FILE),
    },
    {
      id: "engine-chromium",
      title: "Recording browser",
      summary: "Credits and licences of the Chromium build that records takes and exports videos",
      kind: "file",
      path: path.join(engine, CHROMIUM_CREDITS_FILE),
    },
    {
      id: "ffmpeg",
      title: "FFmpeg",
      summary: "GNU General Public License v3. Separate programs that encode and mix the video.",
      kind: "text",
      path: path.join(engine, "ffmpeg", "LICENSE.txt"),
    },
  ];
}

async function entryFor({
  kind,
  path: file,
  ...entry
}: LicenceSource): Promise<LicenceEntry | null> {
  if (kind === "file") return existsSync(file) ? { ...entry, file } : null;
  const text = await readFile(file, "utf8").catch(() => "");
  return text ? { ...entry, text } : null;
}

const openableFiles = () => new Set(sources().flatMap((s) => (s.kind === "file" ? [s.path] : [])));

export async function listLicences(): Promise<LicenceEntry[]> {
  const entries = await Promise.all(sources().map(entryFor));
  return entries.filter((e) => e !== null);
}

export async function openLicenceFile(file: string): Promise<void> {
  if (!openableFiles().has(file)) throw new Error("That file isn't one of the licence files.");
  const error = await shell.openPath(file);
  if (error) throw new Error(error);
}
