import { randomUUID } from "node:crypto";
import path from "node:path";
import { BrowserWindow, ipcMain, shell, type IpcMainInvokeEvent } from "electron";
import { INVOKE, IPC, type DemoflowApi, type InvokeName, type RunRequest } from "../shared/ipc";
import type { MenuCommand } from "../shared/menu";
import { readLastRun } from "../engine/project-paths";
import { readTrace } from "../traces/store";
import { analyze } from "./analysis";
import { apiRunEvent } from "./api-editor";
import { getApiStatus } from "./api";
import { broadcast, focusMainWindow, getMainWindow, isTrusted, windowState } from "./app-windows";
import { pickInWindow } from "./browser-windows";
import { track } from "./exit";
import {
  addInspectedFonts,
  copyFontToProject,
  inspectFontFiles,
  inspectInstalledFonts,
  listLibrary,
  removeLibraryFont,
} from "./fonts";
import { downloadOnlineFont, fontCatalog } from "./fonts-online";
import { importAssets } from "./import-assets";
import { listLicences, openLicenceFile } from "./licences";
import { runWindowRole } from "./menu";
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
  projectLocations,
  saveWorkflowFile,
  watchProject,
} from "./projects";
import { reportRunEvent, runProgressFor } from "./run-reporter";
import { listSecrets } from "./secrets";
import { patchSettings, readSettings } from "./settings";
import { openSettingsWindow } from "./settings-window";
import { listStills, requestStill } from "./stills";
import { activeRunId, cancelRun, environment, installEngine, isRunning, startRun } from "./workers";

type InvokeHandlers = {
  [K in InvokeName]: (
    event: IpcMainInvokeEvent,
    ...args: Parameters<DemoflowApi[K]>
  ) => ReturnType<DemoflowApi[K]> | Awaited<ReturnType<DemoflowApi[K]>>;
};

type IpcHandler = (event: IpcMainInvokeEvent, ...args: never[]) => unknown;

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
  if (isRunning()) throw new Error("A run is already in progress.");
  projectDir = request.projectDir;
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

async function useFontInProject(dir: string, fontId: string) {
  const def = (await listLibrary()).find((f) => f.id === fontId);
  if (!def) throw new Error("That font isn't in your library any more.");
  return copyFontToProject(dir, def);
}

async function revealInFolder(dir: string, rel?: string): Promise<void> {
  const target = rel ? path.join(dir, rel) : dir;
  if (rel) shell.showItemInFolder(target);
  else await shell.openPath(target);
}

const HANDLERS: InvokeHandlers = {
  listRecentProjects: plain(listRecent),
  createProject: async (_event, opts) => remember(await createProjectDir(getMainWindow(), opts)),
  openProject: async (_event, dir) => remember(await openProjectDir(getMainWindow(), dir)),
  saveWorkflow: plain(saveWorkflowFile),
  projectLocations: plain(projectLocations),
  chooseFolder: (event, opts) => chooseFolder(senderWindow(event), opts),
  watchProject: (_event, dir) =>
    watchProject(dir, (change) =>
      getMainWindow()?.webContents.send(IPC.projectFileChanged, change),
    ),
  readLastRun: plain(readLastRun),
  importAssets: (_event, dir, paths) => importAssets(getMainWindow(), dir, paths),
  revealInFolder: plain(revealInFolder),
  runWorkflow: plain(startWorkflowRun),
  cancelRun: plain(cancelRun),
  runStatus: () => {
    const runId = activeRunId();
    return {
      runId,
      projectDir: isRunning() ? projectDir : null,
      progress: runProgressFor(runId),
    };
  },
  pickElement: (_event, request) => pickInWindow(getMainWindow(), request),
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
  inspectFontFiles: (_event, paths) => inspectFontFiles(getMainWindow(), paths),
  inspectInstalledFonts: plain(inspectInstalledFonts),
  addInspectedFonts: plain(addInspectedFonts),
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

export function registerIpc(): void {
  const channels: Record<string, string> = INVOKE;
  const handlers: Record<string, IpcHandler> = HANDLERS;
  for (const [name, handler] of Object.entries(handlers))
    ipcMain.handle(channels[name], async (event, ...args) => {
      assertTrustedSender(event);
      return handler(event, ...(args as never[]));
    });
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
