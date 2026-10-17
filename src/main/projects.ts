import { existsSync, watch, type FSWatcher } from "node:fs";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { app, type BrowserWindow } from "electron";
import { z } from "zod";
import type {
  OpenProject,
  ProjectFileChange,
  ProjectLocations,
  RecentProject,
  SaveResult,
} from "../shared/ipc";
import { expandHome, projectFolderName } from "../shared/project-folder";
import { PROJECTS_FOLDER_NAME } from "../shared/settings";
import { WORKFLOW_FILE, workflowSchema, type Workflow } from "../shared/schema/workflow";
import { getTemplate } from "../shared/templates";
import { debounce } from "../shared/timing";
import { errorMessage } from "../shared/errors";
import { sha256Hex } from "../engine/hash";
import { atomicWrite, isMissingFile, readJsonKeepingBroken, writeJson } from "../engine/json";
import { projectPaths, readLastRun } from "../engine/project-paths";
import { PROJECT_DIRS } from "../shared/project-dirs";
import { openDialog } from "./dialogs";
import { ensureSecretsDir } from "./secrets";
import { patchSettings, readSettings } from "./settings";
import { tryParseJson } from "../shared/guards";

const MAX_RECENT = 20;

const recentSchema = z.object({
  dir: z.string(),
  name: z.string(),
  openedAt: z.number(),
}) satisfies z.ZodType<RecentProject>;

const recentFile = () => path.join(app.getPath("userData"), "recent-projects.json");

const existingDir = (dir?: string) => (dir && existsSync(dir) ? dir : undefined);

async function readRecent(): Promise<RecentProject[] | null> {
  try {
    const stored = await readJsonKeepingBroken(recentFile(), z.array(z.unknown()));
    return (stored ?? []).flatMap((entry) => {
      const parsed = recentSchema.safeParse(entry);
      return parsed.success && existsSync(path.join(parsed.data.dir, WORKFLOW_FILE))
        ? [parsed.data]
        : [];
    });
  } catch (error) {
    process.stderr.write(`[projects] recent projects: ${errorMessage(error)}\n`);
    return null;
  }
}

export const listRecent = async (): Promise<RecentProject[]> => (await readRecent()) ?? [];

async function touchRecent(dir: string, name: string): Promise<void> {
  const list = await readRecent();
  if (!list) return;
  const next = [{ dir, name, openedAt: Date.now() }, ...list.filter((p) => p.dir !== dir)];
  await mkdir(path.dirname(recentFile()), { recursive: true });
  await writeJson(recentFile(), next.slice(0, MAX_RECENT));
}

export async function projectLocations(): Promise<ProjectLocations> {
  const { projects } = await readSettings();
  const defaultDir = projects.defaultDir || path.join(app.getPath("home"), PROJECTS_FOLDER_NAME);
  return { defaultDir, suggestedDir: existingDir(projects.lastDir) ?? defaultDir };
}

export async function chooseFolder(
  win: BrowserWindow | null,
  opts: { title: string; defaultPath?: string },
): Promise<string | null> {
  const [folder] = await openDialog(win, {
    title: opts.title,
    defaultPath: existingDir(opts.defaultPath),
    properties: ["openDirectory", "createDirectory"],
  });
  return folder || null;
}

const versionOf = (text: string) => sha256Hex(text, 16);

const written = new Map<string, string>();

async function readProjectFile(dir: string): Promise<{ text: string; version: string } | null> {
  try {
    const text = await readFile(path.join(dir, WORKFLOW_FILE), "utf8");
    return { text, version: versionOf(text) };
  } catch (error) {
    if (isMissingFile(error)) return null;
    throw error;
  }
}

type ParsedWorkflow = { ok: true; workflow: Workflow } | { ok: false; problem: string };

function parseWorkflowText(text: string): ParsedWorkflow {
  const json = tryParseJson(text);
  if (!json.ok) return { ok: false, problem: "the file is damaged" };
  const parsed = workflowSchema.safeParse(json.value);
  if (parsed.success) return { ok: true, workflow: parsed.data };
  const issue = parsed.error.issues[0];
  const where = issue.path.length ? `${issue.path.join(".")}: ` : "";
  return { ok: false, problem: `${where}${issue.message}` };
}

const workflowOrNull = (text: string): Workflow | null => {
  const parsed = parseWorkflowText(text);
  return parsed.ok ? parsed.workflow : null;
};

export async function pickProjectDir(win: BrowserWindow | null): Promise<string | null> {
  const [folder] = await openDialog(win, {
    title: "Open Demoflow project",
    defaultPath: existingDir((await projectLocations()).defaultDir),
    properties: ["openDirectory"],
  });
  return folder || null;
}

export async function openProjectDir(target: string): Promise<OpenProject> {
  const file = await readProjectFile(target);
  if (!file) throw new Error(`There's no Demoflow project in ${target}.`);
  const parsed = parseWorkflowText(file.text);
  if (!parsed.ok) throw new Error(`The project in ${target} can't be read (${parsed.problem}).`);
  const { workflow } = parsed;
  written.set(target, file.version);
  await touchRecent(target, workflow.name);
  return {
    dir: target,
    workflow,
    lastRun: await readLastRun(target),
    version: file.version,
  };
}

export async function createProjectDir(
  win: BrowserWindow | null,
  opts: { name: string; template: string; dir?: string },
): Promise<OpenProject | null> {
  const chosen =
    opts.dir ??
    (await chooseFolder(win, {
      title: "Choose where to create the project",
      defaultPath: (await projectLocations()).suggestedDir,
    }));
  if (!chosen) return null;
  const parent = expandHome(chosen, app.getPath("home"));
  if (!path.isAbsolute(parent)) throw new Error("Choose a full folder path for the project.");
  const dir = path.join(parent, projectFolderName(opts.name));
  if (existsSync(path.join(dir, WORKFLOW_FILE)))
    throw new Error(`${dir} already contains a project.`);
  await mkdir(path.join(dir, "assets"), { recursive: true });
  await ensureSecretsDir(dir);
  await writeFile(path.join(dir, ".gitignore"), `${PROJECT_DIRS.meta}/\n`);
  const workflow = getTemplate(opts.template).build(opts.name.trim() || "Demo");
  const version = await writeWorkflowFile(dir, workflow);
  await patchSettings({ projects: { lastDir: parent } });
  return { dir, workflow, lastRun: null, version };
}

async function writeWorkflowFile(dir: string, workflow: Workflow): Promise<string> {
  const valid = workflowSchema.parse(workflow);
  const text = `${JSON.stringify(valid, null, 2)}\n`;
  const version = versionOf(text);
  written.set(dir, version);
  await atomicWrite(path.join(dir, WORKFLOW_FILE), text);
  await touchRecent(dir, valid.name);
  return version;
}

export async function saveWorkflowFile(
  dir: string,
  workflow: Workflow,
  base?: string | null,
): Promise<SaveResult> {
  if (base !== undefined) {
    const current = await readProjectFile(dir);
    if (current && current.version !== base)
      return { ok: false, version: current.version, workflow: workflowOrNull(current.text) };
  }
  return { ok: true, version: await writeWorkflowFile(dir, workflow) };
}

const logWatchError = (dir: string, error: unknown) =>
  process.stderr.write(`[projects] watching ${dir}: ${errorMessage(error)}\n`);

let watcher: FSWatcher | null = null;
let watchedDir: string | null = null;

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
    announced = current.version;
    notify({ dir, workflow: workflowOrNull(current.text), version: current.version });
  };
  const checkSoon = debounce(
    () => void check().catch((error: unknown) => logWatchError(dir, error)),
    150,
  );
  try {
    watcher = watch(dir, (_event, name) => {
      if (name === WORKFLOW_FILE) checkSoon();
    });
    watcher.on("error", (error) => logWatchError(dir, error));
  } catch (error) {
    logWatchError(dir, error);
  }
}

export async function clearCache(dir: string): Promise<void> {
  await rm(projectPaths(dir).cache, { recursive: true, force: true });
  await rm(projectPaths(dir).lastRun, { force: true });
}
