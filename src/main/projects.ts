import { createHash } from "node:crypto";
import { existsSync, watch, type FSWatcher } from "node:fs";
import { copyFile, mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { app, dialog, type BrowserWindow } from "electron";
import type {
  OpenProject,
  ProjectFileChange,
  ProjectLocations,
  RecentProject,
  SaveResult,
} from "../shared/ipc";
import { projectFolderName } from "../shared/project-folder";
import { PROJECTS_FOLDER_NAME } from "../shared/settings";
import { patchSettings, readSettings } from "./settings";
import type { LastRun } from "../shared/schema/values";
import { WORKFLOW_FILE, workflowSchema, type Workflow } from "../shared/schema/workflow";
import { getTemplate } from "../shared/templates";

const recentFile = () => path.join(app.getPath("userData"), "recent-projects.json");

export async function listRecent(): Promise<RecentProject[]> {
  try {
    const list = JSON.parse(await readFile(recentFile(), "utf8")) as RecentProject[];
    return list.filter((p) => existsSync(path.join(p.dir, WORKFLOW_FILE)));
  } catch {
    return [];
  }
}

async function touchRecent(dir: string, name: string): Promise<void> {
  const list = (await listRecent()).filter((p) => p.dir !== dir);
  list.unshift({ dir, name, openedAt: Date.now() });
  await mkdir(path.dirname(recentFile()), { recursive: true });
  await writeFile(recentFile(), JSON.stringify(list.slice(0, 20), null, 2));
}

async function readLastRunFile(dir: string): Promise<LastRun | null> {
  try {
    return JSON.parse(
      await readFile(path.join(dir, ".demoflow", "last-run.json"), "utf8"),
    ) as LastRun;
  } catch {
    return null;
  }
}

export { readLastRunFile as readLastRun };

export async function projectsDir(): Promise<string> {
  const { projects } = await readSettings();
  return projects.defaultDir || path.join(app.getPath("home"), PROJECTS_FOLDER_NAME);
}

export async function projectLocations(): Promise<ProjectLocations> {
  const defaultDir = await projectsDir();
  const { projects } = await readSettings();
  const last = projects.lastDir && existsSync(projects.lastDir) ? projects.lastDir : "";
  return { defaultDir, suggestedDir: last || defaultDir };
}

export async function chooseFolder(
  win: BrowserWindow | null,
  opts: { title: string; defaultPath?: string },
): Promise<string | null> {
  const options = {
    title: opts.title,
    defaultPath: opts.defaultPath && existsSync(opts.defaultPath) ? opts.defaultPath : undefined,
    properties: ["openDirectory", "createDirectory"] as Array<"openDirectory" | "createDirectory">,
  };
  const res = win
    ? await dialog.showOpenDialog(win, options)
    : await dialog.showOpenDialog(options);
  return res.canceled || !res.filePaths[0] ? null : res.filePaths[0];
}

const versionOf = (text: string) => createHash("sha256").update(text).digest("hex").slice(0, 16);

const written = new Map<string, string>();

async function readProjectFile(dir: string): Promise<{ text: string; version: string } | null> {
  try {
    const text = await readFile(path.join(dir, WORKFLOW_FILE), "utf8");
    return { text, version: versionOf(text) };
  } catch {
    return null;
  }
}

export async function openProjectDir(
  win: BrowserWindow | null,
  dir?: string,
): Promise<OpenProject | null> {
  let target = dir;
  if (!target) {
    const options = {
      title: "Open Demoflow project",
      defaultPath: existingDir(await projectsDir()),
      properties: ["openDirectory"] as Array<"openDirectory">,
    };
    const res = win
      ? await dialog.showOpenDialog(win, options)
      : await dialog.showOpenDialog(options);
    if (res.canceled || !res.filePaths[0]) return null;
    target = res.filePaths[0];
  }
  const file = await readProjectFile(target);
  if (!file) throw new Error(`There's no Demoflow project in ${target}.`);
  const workflow = workflowSchema.parse(JSON.parse(file.text));
  written.set(target, file.version);
  await touchRecent(target, workflow.name);
  return {
    dir: target,
    workflow,
    lastRun: await readLastRunFile(target),
    version: file.version,
  };
}

const existingDir = (dir: string) => (existsSync(dir) ? dir : undefined);

export async function createProjectDir(
  win: BrowserWindow | null,
  opts: { name: string; template: string; dir?: string },
): Promise<OpenProject | null> {
  const parent =
    opts.dir ??
    (await chooseFolder(win, {
      title: "Choose where to create the project",
      defaultPath: (await projectLocations()).suggestedDir,
    }));
  if (!parent) return null;
  const home = app.getPath("home");
  const expanded = parent === "~" ? home : parent.replace(/^~(?=[\\/])/, home);
  if (!path.isAbsolute(expanded)) throw new Error("Choose a full folder path for the project.");
  const dir = path.join(expanded, projectFolderName(opts.name));
  if (existsSync(path.join(dir, WORKFLOW_FILE)))
    throw new Error(`${dir} already contains a project.`);
  await mkdir(path.join(dir, "assets"), { recursive: true });
  await mkdir(path.join(dir, "secrets"), { recursive: true });
  await writeFile(path.join(dir, "secrets", ".gitignore"), "*\n!.gitignore\n");
  await writeFile(path.join(dir, ".gitignore"), ".demoflow/\nsecrets/*\n!secrets/.gitignore\n");
  const workflow = getTemplate(opts.template).build(opts.name.trim() || "Demo");
  const saved = await saveWorkflowFile(dir, workflow);
  await patchSettings({ projects: { lastDir: expanded } });
  return { dir, workflow, lastRun: null, version: saved.ok ? saved.version : null };
}

export async function saveWorkflowFile(
  dir: string,
  workflow: Workflow,
  base?: string | null,
): Promise<SaveResult> {
  const valid = workflowSchema.parse(workflow);
  if (base !== undefined) {
    const current = await readProjectFile(dir);
    if (current && current.version !== base) return conflict(current);
  }
  const file = path.join(dir, WORKFLOW_FILE);
  const tmp = `${file}.tmp`;
  const text = `${JSON.stringify(valid, null, 2)}\n`;
  const version = versionOf(text);
  written.set(dir, version);
  await writeFile(tmp, text);
  await rename(tmp, file);
  await touchRecent(dir, valid.name);
  return { ok: true, version };
}

function conflict(current: { text: string; version: string }): SaveResult {
  try {
    return {
      ok: false,
      version: current.version,
      workflow: workflowSchema.parse(JSON.parse(current.text)),
    };
  } catch {
    return { ok: false, version: current.version, workflow: null };
  }
}

let watcher: FSWatcher | null = null;
let watchedDir: string | null = null;
let settle: ReturnType<typeof setTimeout> | null = null;

export function watchProject(
  dir: string | null,
  notify: (change: ProjectFileChange) => void,
): void {
  if (dir === watchedDir) return;
  watcher?.close();
  watcher = null;
  watchedDir = dir;
  if (!dir) return;
  let announced = written.get(dir) ?? null;
  const check = async () => {
    const current = await readProjectFile(dir);
    if (!current || watchedDir !== dir) return;
    if (current.version === written.get(dir) || current.version === announced) return;
    try {
      const workflow = workflowSchema.parse(JSON.parse(current.text));
      announced = current.version;
      notify({ dir, workflow, version: current.version });
    } catch {
      return;
    }
  };
  try {
    watcher = watch(dir, (_event, name) => {
      if (name !== WORKFLOW_FILE) return;
      if (settle) clearTimeout(settle);
      settle = setTimeout(() => void check(), 150);
    });
  } catch {
    watcher = null;
  }
}

export async function importAssets(
  win: BrowserWindow | null,
  dir: string,
  paths?: string[],
): Promise<string[]> {
  let files = paths;
  if (!files?.length) {
    const options = {
      title: "Import media",
      properties: ["openFile", "multiSelections"] as Array<"openFile" | "multiSelections">,
      filters: [
        {
          name: "Media",
          extensions: [
            "mp4",
            "mov",
            "webm",
            "mkv",
            "png",
            "jpg",
            "jpeg",
            "webp",
            "gif",
            "svg",
            "mp3",
            "wav",
            "m4a",
            "aac",
            "ogg",
            "flac",
            "lottie",
            "json",
          ],
        },
        { name: "All files", extensions: ["*"] },
      ],
    };
    const res = win
      ? await dialog.showOpenDialog(win, options)
      : await dialog.showOpenDialog(options);
    if (res.canceled) return [];
    files = res.filePaths;
  }
  const assets = path.join(dir, "assets");
  await mkdir(assets, { recursive: true });
  const out: string[] = [];
  for (const file of files) {
    const relInProject = path.relative(dir, file);
    if (!relInProject.startsWith("..") && !path.isAbsolute(relInProject)) {
      out.push(relInProject.split(path.sep).join("/"));
      continue;
    }
    let name = path.basename(file);
    let n = 1;
    while (existsSync(path.join(assets, name))) {
      const ext = path.extname(file);
      name = `${path.basename(file, ext)}-${n++}${ext}`;
    }
    await copyFile(file, path.join(assets, name));
    out.push(`assets/${name}`);
  }
  return out;
}

export async function clearCache(dir: string): Promise<void> {
  await rm(path.join(dir, ".demoflow", "cache"), { recursive: true, force: true });
  await rm(path.join(dir, ".demoflow", "last-run.json"), { force: true });
}
