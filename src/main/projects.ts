import { existsSync, watch, type FSWatcher } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { firstIssueText } from "@opsis/core/api/issue-text";
import { nonBlankString, parseJsonWith } from "@opsis/core/guards";
import type {
  OpenProject,
  ProjectFileChange,
  ProjectFilesChange,
  ProjectLocations,
  RecentProject,
  SaveResult,
} from "@opsis/core/ipc";
import { PROJECT_DIRS } from "@opsis/core/project-dirs";
import { expandHome, projectFolderName } from "@opsis/core/project-folder";
import { WORKFLOW_FILE, workflowSchema, type Workflow } from "@opsis/core/schema/workflow";
import { PROJECTS_FOLDER_NAME } from "@opsis/core/settings";
import { getTemplate } from "@opsis/core/templates";
import { debounce } from "@opsis/core/timing";
import { desc } from "drizzle-orm";
import { app, type BrowserWindow, type OpenDialogOptions } from "electron";
import { sha256Hex } from "../engine/hash";
import { atomicWrite, readTextIfPresent } from "../engine/json";
import { readLastRun } from "../engine/last-run";
import { logError } from "../engine/log";
import { projectPaths } from "../engine/project-paths";
import { db } from "./db";
import { projects } from "./db/schema";
import { openDialog } from "./dialogs";
import { watchLoadedFiles, type LoadedFilesWatch } from "./loaded-files-watch";
import { ensureSecretsDir } from "./secrets";
import { patchSettings, readSettings } from "./settings";
import { forgetViewState, readViewState } from "./view-state";

const MAX_RECENT = 20;

const existingDir = (dir?: string) => (dir && existsSync(dir) ? dir : undefined);

export const listRecent = (): RecentProject[] =>
  db()
    .select({ dir: projects.dir, name: projects.name, openedAt: projects.openedAt })
    .from(projects)
    .orderBy(desc(projects.openedAt))
    .all()
    .filter((p) => existsSync(path.join(p.dir, WORKFLOW_FILE)))
    .slice(0, MAX_RECENT);

function touchRecent(dir: string, name: string): void {
  const openedAt = Date.now();
  db()
    .insert(projects)
    .values({ dir, name, openedAt })
    .onConflictDoUpdate({ target: projects.dir, set: { name, openedAt } })
    .run();
}

export function projectLocations(): ProjectLocations {
  const { projects: folders } = readSettings();
  const defaultDir = folders.defaultDir || path.join(app.getPath("home"), PROJECTS_FOLDER_NAME);
  return { defaultDir, suggestedDir: existingDir(folders.lastDir) ?? defaultDir };
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
  const { defaultDir } = projectLocations();
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
  touchRecent(target, workflow.name);
  openDir = target;
  return {
    dir: target,
    workflow,
    lastRun: await readLastRun(target),
    version: file.version,
    view: readViewState(target),
  };
}

async function writeWorkflowFile(dir: string, workflow: Workflow): Promise<string> {
  const normalized = workflowSchema.parse(workflow);
  const text = `${JSON.stringify(normalized, null, 2)}\n`;
  const version = versionOf(text);
  written.set(dir, version);
  await atomicWrite(path.join(dir, WORKFLOW_FILE), text);
  touchRecent(dir, normalized.name);
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
      defaultPath: projectLocations().suggestedDir,
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
  forgetViewState(dir);
  const version = await writeWorkflowFile(dir, workflow);
  patchSettings({ projects: { lastDir: parent } });
  return { dir, workflow, lastRun: null, version, view: {} };
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
let loadedFiles: LoadedFilesWatch | null = null;
let loadedRefs: { dir: string; refs: readonly string[] } | null = null;
let watchedDir: string | null = null;

export function watchProjectFiles(dir: string, refs: readonly string[]): void {
  loadedRefs = { dir, refs };
  if (dir === watchedDir) loadedFiles?.follow(refs);
}

export function watchProject(
  dir: string | null,
  notify: (change: ProjectFileChange) => void,
  notifyFiles: (change: ProjectFilesChange) => void,
): void {
  if (dir === watchedDir) return;
  watcher?.close();
  watcher = null;
  loadedFiles?.close();
  loadedFiles = null;
  watchedDir = dir;
  if (!dir) return;
  loadedFiles = watchLoadedFiles(dir, notifyFiles);
  if (loadedRefs?.dir === dir) loadedFiles.follow(loadedRefs.refs);
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
