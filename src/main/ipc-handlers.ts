import { randomUUID } from "node:crypto";
import path from "node:path";
import { BrowserWindow, ipcMain, shell, type IpcMainInvokeEvent } from "electron";
import { readLastRun } from "../engine/last-run";
import { INVOKE, IPC, type DemoflowApi, type InvokeName, type RunRequest } from "../shared/ipc";
import { findById } from "../shared/lists";
import type { MenuCommand } from "../shared/menu";
import { keysOf } from "../shared/nodes/params";
import { readTrace } from "../traces/store";
import { analyze } from "./analysis";
import { getApiStatus } from "./api";
import { apiRunEvent } from "./api-editor";
import { broadcast, isTrusted } from "./app-windows";
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
import { activeRun, cancelRun, startRun } from "./run-host";
import { reportRunEvent, runProgressFor } from "./run-reporter";
import { listSecrets } from "./secrets";
import { patchSettings, readSettings } from "./settings";
import { openSettingsWindow } from "./settings-window";
import { listStills, requestStill } from "./stills";
import { focusMainWindow, getMainWindow, sendMenuCommand, windowState } from "./window-registry";
import { runWindowRole } from "./window-roles";

type Result<K extends InvokeName> =
  | ReturnType<DemoflowApi[K]>
  | Awaited<ReturnType<DemoflowApi[K]>>;

type Args<K extends InvokeName> = Parameters<DemoflowApi[K]>;

type Handler<K extends InvokeName> =
  | ((...args: Args<K>) => Result<K>)
  | { withWindow: (win: BrowserWindow | null, ...args: Args<K>) => Result<K> };

async function startWorkflowRun(request: RunRequest) {
  const runId = randomUUID();
  const live = await startRun(runId, request, (event) => {
    getMainWindow()?.webContents.send(IPC.runEvent, event);
    apiRunEvent(event);
    reportRunEvent(event);
  });
  const viewer = panelWindow("run") ?? getMainWindow();
  if (live) viewer?.webContents.postMessage(IPC.livePort, { runId }, [live]);
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
  if (rel) shell.showItemInFolder(path.join(dir, rel));
  else await shell.openPath(dir);
}

const HANDLERS: { [K in InvokeName]: Handler<K> } = {
  createProject: { withWindow: createProjectDir },
  pickProjectDir: { withWindow: pickProjectDir },
  chooseFolder: { withWindow: chooseFolder },
  importAssets: { withWindow: importAssets },
  pickElement: { withWindow: pickInWindow },
  inspectFontFiles: { withWindow: inspectFontFiles },
  windowRole: {
    withWindow: (win, role) => {
      if (win) runWindowRole(win, role);
    },
  },
  getWindowState: { withWindow: windowState },
  listRecentProjects: listRecent,
  openProject: openProjectDir,
  saveWorkflow: saveWorkflowFile,
  projectLocations,
  watchProject: (dir) =>
    watchProject(dir, (change) =>
      getMainWindow()?.webContents.send(IPC.projectFileChanged, change),
    ),
  readLastRun,
  revealInFolder,
  runWorkflow: startWorkflowRun,
  cancelRun,
  runStatus,
  getEnvironment: environment,
  installEngine: () =>
    track(
      "The engine is being set up.",
      installEngine((e) => broadcast(IPC.engineEvent, e)),
    ),
  getSettings: readSettings,
  patchSettings,
  openSettings: openSettingsWindow,
  listLicences,
  openLicenceFile,
  apiStatus: getApiStatus,
  readTrace,
  listSecrets,
  listFonts: listLibrary,
  inspectInstalledFonts,
  addInspectedFonts,
  releaseInspectedFonts,
  removeFont: removeLibraryFont,
  fontCatalog,
  downloadFont: (id) => track("A font is being downloaded.", downloadOnlineFont(id)),
  useFontInProject,
  clearCache,
  requestStill,
  listStills,
  analyze,
  openPanelWindow,
  focusPanelWindow,
  closePanelWindow,
  closePanelWindows,
  listPanelWindows: openPanels,
};

function assertTrustedSender(event: IpcMainInvokeEvent): void {
  if (isTrusted(event.sender.id)) return;
  throw new Error("Blocked IPC call from an unknown renderer.");
}

function handle<K extends InvokeName>(name: K): void {
  const handler: Handler<K> = HANDLERS[name];
  ipcMain.handle(INVOKE[name], async (event, ...args: Args<K>) => {
    assertTrustedSender(event);
    return typeof handler === "function"
      ? handler(...args)
      : handler.withWindow(BrowserWindow.fromWebContents(event.sender), ...args);
  });
}

export function registerIpc(): void {
  for (const name of keysOf(HANDLERS)) handle(name);
  ipcMain.on(IPC.menuToEditor, (event, command: MenuCommand) => {
    if (!isTrusted(event.sender.id)) return;
    focusMainWindow();
    sendMenuCommand(command);
  });
  ipcMain.on(IPC.setTitle, (event, title: string) => {
    if (isTrusted(event.sender.id)) BrowserWindow.fromWebContents(event.sender)?.setTitle(title);
  });
}
