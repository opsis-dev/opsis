import { existsSync, watch, type FSWatcher } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { app, type BrowserWindow, type OpenDialogOptions } from "electron";
import { z } from "zod";
import { sha256Hex } from "../engine/hash";
import { atomicWrite, readJsonKeepingBroken, readTextIfPresent, writeJson } from "../engine/json";
import { readLastRun } from "../engine/last-run";
import { logError } from "../engine/log";
import { projectPaths } from "../engine/project-paths";
import { firstIssueText } from "../shared/api/issue-text";
import { nonBlankString, parseJsonWith } from "../shared/guards";
import type {
  OpenProject,
  ProjectFileChange,
  ProjectLocations,
  RecentProject,
  SaveResult,
} from "../shared/ipc";
import { PROJECT_DIRS } from "../shared/project-dirs";
import { expandHome, projectFolderName } from "../shared/project-folder";
import { WORKFLOW_FILE, workflowSchema, type Workflow } from "../shared/schema/workflow";
import { PROJECTS_FOLDER_NAME } from "../shared/settings";
import { getTemplate } from "../shared/templates";
import { debounce } from "../shared/timing";
import { openDialog } from "./dialogs";
import { ensureSecretsDir } from "./secrets";
import { patchSettings, readSettings } from "./settings";

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
    logError("projects", error, "recent projects");
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
  properties: OpenDialogOptions["properties"] = ["openDirectory", "createDirectory"],
): Promise<string | null> {
  const [folder] = await openDialog(win, {
    title: opts.title,
    defaultPath: existingDir(opts.defaultPath),
    properties,
  });
  return folder || null;
}

const versionOf = (text: string) => sha256Hex(text, 16);

const written = new Map<string, string>();

async function readProjectFile(dir: string): Promise<{ text: string; version: string } | null> {
  const text = await readTextIfPresent(path.join(dir, WORKFLOW_FILE));
  return text === null ? null : { text, version: versionOf(text) };
}

type ParsedWorkflow = { ok: true; workflow: Workflow } | { ok: false; problem: string };

function parseWorkflowText(text: string): ParsedWorkflow {
  const parsed = parseJsonWith(workflowSchema, text);
  if (!parsed) return { ok: false, problem: "the file is damaged" };
  if (parsed.success) return { ok: true, workflow: parsed.data };
  return { ok: false, problem: firstIssueText(parsed.error) };
}

const workflowOrNull = (text: string): Workflow | null => {
  const parsed = parseWorkflowText(text);
  return parsed.ok ? parsed.workflow : null;
};

export async function pickProjectDir(win: BrowserWindow | null): Promise<string | null> {
  const { defaultDir } = await projectLocations();
  return chooseFolder(win, { title: "Open Opsis project", defaultPath: defaultDir }, [
    "openDirectory",
  ]);
}

let openDir: string | null = null;

export const currentProjectDir = (): string | null => openDir;

export async function openProjectDir(target: string): Promise<OpenProject> {
  const file = await readProjectFile(target);
  if (!file) throw new Error(`There's no Opsis project in ${target}.`);
  const parsed = parseWorkflowText(file.text);
  if (!parsed.ok) throw new Error(`The project in ${target} can't be read (${parsed.problem}).`);
  const { workflow } = parsed;
  written.set(target, file.version);
  await touchRecent(target, workflow.name);
  openDir = target;
  return {
    dir: target,
    workflow,
    lastRun: await readLastRun(target),
    version: file.version,
  };
}

async function writeWorkflowFile(dir: string, workflow: Workflow): Promise<string> {
  const normalized = workflowSchema.parse(workflow);
  const text = `${JSON.stringify(normalized, null, 2)}\n`;
  const version = versionOf(text);
  written.set(dir, version);
  await atomicWrite(path.join(dir, WORKFLOW_FILE), text);
  await touchRecent(dir, normalized.name);
  return version;
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
  const workflow = getTemplate(opts.template).build(nonBlankString(opts.name) ?? "Demo");
  const version = await writeWorkflowFile(dir, workflow);
  await patchSettings({ projects: { lastDir: parent } });
  return { dir, workflow, lastRun: null, version };
}

export async function saveWorkflowFile(
  dir: string,
  workflow: Workflow,
  base: string | null,
): Promise<SaveResult> {
  const current = await readProjectFile(dir);
  if (current && current.version !== base)
    return { ok: false, version: current.version, workflow: workflowOrNull(current.text) };
  return { ok: true, version: await writeWorkflowFile(dir, workflow) };
}

const logWatchError = (dir: string, error: unknown) =>
  logError("projects", error, `watching ${dir}`);

const WATCH_DEBOUNCE_MS = 150;

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
    WATCH_DEBOUNCE_MS,
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
