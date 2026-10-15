import { existsSync } from "node:fs";
import { copyFile, mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { app, dialog, type BrowserWindow } from "electron";
import type { OpenProject, RecentProject } from "../shared/ipc";
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

export async function openProjectDir(
  win: BrowserWindow | null,
  dir?: string,
): Promise<OpenProject | null> {
  let target = dir;
  if (!target) {
    const options = {
      title: "Open Demoflow project",
      properties: ["openDirectory"] as Array<"openDirectory">,
    };
    const res = win
      ? await dialog.showOpenDialog(win, options)
      : await dialog.showOpenDialog(options);
    if (res.canceled || !res.filePaths[0]) return null;
    target = res.filePaths[0];
  }
  const file = path.join(target, WORKFLOW_FILE);
  if (!existsSync(file)) throw new Error(`There's no Demoflow project in ${target}.`);
  const workflow = workflowSchema.parse(JSON.parse(await readFile(file, "utf8")));
  await touchRecent(target, workflow.name);
  return { dir: target, workflow, lastRun: await readLastRunFile(target) };
}

export async function createProjectDir(
  win: BrowserWindow | null,
  opts: { name: string; template: string; dir?: string },
): Promise<OpenProject | null> {
  let parent = opts.dir;
  if (!parent) {
    const options = {
      title: "Choose where to create the project",
      properties: ["openDirectory", "createDirectory"] as Array<
        "openDirectory" | "createDirectory"
      >,
    };
    const res = win
      ? await dialog.showOpenDialog(win, options)
      : await dialog.showOpenDialog(options);
    if (res.canceled || !res.filePaths[0]) return null;
    parent = res.filePaths[0];
  }
  const folder =
    opts.name
      .trim()
      .replace(/[^\w.-]+/g, "-")
      .toLowerCase() || "demo";
  const dir = path.join(parent, folder);
  if (existsSync(path.join(dir, WORKFLOW_FILE)))
    throw new Error(`${dir} already contains a project.`);
  await mkdir(path.join(dir, "assets"), { recursive: true });
  await mkdir(path.join(dir, "secrets"), { recursive: true });
  await writeFile(path.join(dir, "secrets", ".gitignore"), "*\n!.gitignore\n");
  await writeFile(path.join(dir, ".gitignore"), ".demoflow/\nsecrets/*\n!secrets/.gitignore\n");
  const workflow = getTemplate(opts.template).build(opts.name.trim() || "Demo");
  await saveWorkflowFile(dir, workflow);
  await touchRecent(dir, workflow.name);
  return { dir, workflow, lastRun: null };
}

export async function saveWorkflowFile(dir: string, workflow: Workflow): Promise<void> {
  const valid = workflowSchema.parse(workflow);
  const file = path.join(dir, WORKFLOW_FILE);
  const tmp = `${file}.tmp`;
  await writeFile(tmp, `${JSON.stringify(valid, null, 2)}\n`);
  await rename(tmp, file);
  await touchRecent(dir, valid.name);
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
