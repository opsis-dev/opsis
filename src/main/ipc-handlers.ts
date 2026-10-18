import { randomUUID } from "node:crypto";
import path from "node:path";
import { BrowserWindow, ipcMain, shell, type IpcMainInvokeEvent } from "electron";
import { INVOKE, IPC, type DemoflowApi, type InvokeName, type RunRequest } from "../shared/ipc";
import type { MenuCommand } from "../shared/menu";
import { keysOf } from "../shared/nodes/params";
import { readLastRun } from "../engine/project-paths";
import { readTrace } from "../traces/store";
import { analyze } from "./analysis";
import { apiRunEvent } from "./api-editor";
import { getApiStatus } from "./api";
import { broadcast, focusMainWindow, getMainWindow, isTrusted, windowState } from "./app-windows";
import { pickInWindow } from "./browser-windows";
import { environment, installEngine } from "./engine-setup";
import { track } from "./exit";
import {
  addInspectedFonts,
  copyFontToProject,
  inspectFontFiles,
  inspectInstalledFonts,
  listLibrary,
  releaseInspectedFonts,
  removeLibraryFont,
} from "./fonts";
import { downloadOnlineFont, fontCatalog } from "./fonts-online";
import { importAssets } from "./import-assets";
import { listLicences, openLicenceFile } from "./licences";
import {
  closePanelWindow,
  closePanelWindows,
  focusPanelWindow,
  openPanels,
  openPanelWindow,
  panelWindow,
} from "./panel-windows";
import {
  chooseFolder,
  clearCache,
  createProjectDir,
  listRecent,
  openProjectDir,
  pickProjectDir,
  projectLocations,
  saveWorkflowFile,
  watchProject,
} from "./projects";
import { reportRunEvent, runProgressFor } from "./run-reporter";
import { listSecrets } from "./secrets";
import { patchSettings, readSettings } from "./settings";
import { openSettingsWindow } from "./settings-window";
import { listStills, requestStill } from "./stills";
import { runWindowRole } from "./window-roles";
import { activeRun, cancelRun, startRun } from "./workers";
import { findById } from "../shared/lists";

type InvokeHandlers = {
  [K in InvokeName]: (
    event: IpcMainInvokeEvent,
    ...args: Parameters<DemoflowApi[K]>
  ) => ReturnType<DemoflowApi[K]> | Awaited<ReturnType<DemoflowApi[K]>>;
};

type ErasedHandler = (event: IpcMainInvokeEvent, ...args: never[]) => unknown;

let projectDir: string | null = null;

export const currentProjectDir = (): string | null => projectDir;

const plain =
  <A extends unknown[], R>(fn: (...args: A) => R) =>
  (_event: IpcMainInvokeEvent, ...args: A): R =>
    fn(...args);

const senderWindow = (event: IpcMainInvokeEvent) => BrowserWindow.fromWebContents(event.sender);

function remember<T extends { dir: string } | null>(project: T): T {
  if (project) projectDir = project.dir;
  return project;
}

async function startWorkflowRun(request: RunRequest) {
  const runId = randomUUID();
  await startRun(
    runId,
    request,
    getMainWindow,
    (event) => {
      getMainWindow()?.webContents.send(IPC.runEvent, event);
      apiRunEvent(event);
      reportRunEvent(event);
    },
    (port) =>
      (panelWindow("run") ?? getMainWindow())?.webContents.postMessage(IPC.livePort, { runId }, [
        port,
      ]),
  );
  return { runId };
}

function runStatus() {
  const run = activeRun();
  const runId = run?.runId ?? null;
  return { runId, projectDir: run?.projectDir ?? null, progress: runProgressFor(runId) };
}

async function useFontInProject(dir: string, fontId: string) {
  const def = findById(await listLibrary(), fontId);
  if (!def) throw new Error("That font isn't in your library any more.");
  return copyFontToProject(dir, def);
}

async function revealInFolder(dir: string, rel?: string): Promise<void> {
  if (rel) {
    shell.showItemInFolder(path.join(dir, rel));
    return;
  }
  await shell.openPath(dir);
}

const HANDLERS: InvokeHandlers = {
  listRecentProjects: plain(listRecent),
  createProject: async (event, opts) => remember(await createProjectDir(senderWindow(event), opts)),
  pickProjectDir: (event) => pickProjectDir(senderWindow(event)),
  openProject: async (_event, dir) => remember(await openProjectDir(dir)),
  saveWorkflow: plain(saveWorkflowFile),
  projectLocations: plain(projectLocations),
  chooseFolder: (event, opts) => chooseFolder(senderWindow(event), opts),
  watchProject: (_event, dir) =>
    watchProject(dir, (change) =>
      getMainWindow()?.webContents.send(IPC.projectFileChanged, change),
    ),
  readLastRun: plain(readLastRun),
  importAssets: (event, dir, paths) => importAssets(senderWindow(event), dir, paths),
  revealInFolder: plain(revealInFolder),
  runWorkflow: plain(startWorkflowRun),
  cancelRun: plain(cancelRun),
  runStatus: plain(runStatus),
  pickElement: (event, request) => pickInWindow(senderWindow(event), request),
  getEnvironment: plain(environment),
  installEngine: () =>
    track(
      "The engine is being set up.",
      installEngine((e) => broadcast(IPC.engineEvent, e)),
    ),
  getSettings: plain(readSettings),
  patchSettings: plain(patchSettings),
  openSettings: plain(openSettingsWindow),
  listLicences: plain(listLicences),
  openLicenceFile: plain(openLicenceFile),
  apiStatus: plain(getApiStatus),
  readTrace: plain(readTrace),
  listSecrets: plain(listSecrets),
  listFonts: plain(listLibrary),
  inspectFontFiles: (event, paths) => inspectFontFiles(senderWindow(event), paths),
  inspectInstalledFonts: plain(inspectInstalledFonts),
  addInspectedFonts: plain(addInspectedFonts),
  releaseInspectedFonts: plain(releaseInspectedFonts),
  removeFont: plain(removeLibraryFont),
  fontCatalog: plain(fontCatalog),
  downloadFont: (_event, id) => track("A font is being downloaded.", downloadOnlineFont(id)),
  useFontInProject: plain(useFontInProject),
  clearCache: plain(clearCache),
  windowRole: (event, role) => {
    const win = senderWindow(event);
    if (win) runWindowRole(win, role);
  },
  getWindowState: (event) => windowState(senderWindow(event)),
  requestStill: plain(requestStill),
  listStills: plain(listStills),
  analyze: plain(analyze),
  openPanelWindow: plain(openPanelWindow),
  focusPanelWindow: plain(focusPanelWindow),
  closePanelWindow: plain(closePanelWindow),
  closePanelWindows: plain(closePanelWindows),
  listPanelWindows: plain(openPanels),
};

function assertTrustedSender(event: IpcMainInvokeEvent): void {
  if (isTrusted(event.sender.id)) return;
  throw new Error("Blocked IPC call from an unknown renderer.");
}

function handleInvoke(name: InvokeName): void {
  const handler: ErasedHandler = HANDLERS[name];
  ipcMain.handle(INVOKE[name], async (event, ...args) => {
    assertTrustedSender(event);
    return handler(event, ...(args as never[]));
  });
}

export function registerIpc(): void {
  for (const name of keysOf(INVOKE)) handleInvoke(name);
  ipcMain.on(IPC.menuToEditor, (event, command: MenuCommand) => {
    const main = getMainWindow();
    if (!isTrusted(event.sender.id) || !main) return;
    focusMainWindow();
    main.webContents.send(IPC.menu, command);
  });
  ipcMain.on(IPC.setTitle, (event, title: string) => {
    if (isTrusted(event.sender.id)) BrowserWindow.fromWebContents(event.sender)?.setTitle(title);
  });
}
