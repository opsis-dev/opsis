import { randomUUID } from "node:crypto";
import path from "node:path";
import {
  app,
  BrowserWindow,
  ipcMain,
  Notification,
  shell,
  type IpcMainInvokeEvent,
} from "electron";
import started from "electron-squirrel-startup";
import { IPC, type PickRequest, type RunRequest } from "../shared/ipc";
import type { AppSettings } from "../shared/settings";
import type { RunEvent } from "../shared/schema/events";
import { pickInWindow } from "./browser-windows";
import type { WindowRole } from "../shared/menu";
import { buildMenu, runWindowRole } from "./menu";
import {
  clearCache,
  createProjectDir,
  importAssets,
  listRecent,
  openProjectDir,
  readLastRun,
  saveWorkflowFile,
} from "./projects";
import { APP_SCHEME, registerProtocolHandlers, registerSchemes } from "./protocol";
import { readSettings, writeSettings } from "./settings";
import { readTrace } from "../traces/store";
import { listSecrets } from "./secrets";
import {
  addInspectedFonts,
  copyFontToProject,
  downloadOnlineFont,
  fontCatalog,
  fontsDir,
  inspectFontFiles,
  inspectInstalledFonts,
  listLibrary,
  removeLibraryFont,
} from "./fonts";
import type { FontInstalledFace, StillRequest, WindowState } from "../shared/ipc";
import { initStills, listStills, requestStill } from "./stills";
import {
  closePanelWindow,
  closePanelWindows,
  focusPanelWindow,
  initPanelWindows,
  isPanelSender,
  openPanels,
  openPanelWindow,
  panelWindow,
} from "./panel-windows";
import type { DockPanelName } from "../shared/ipc";
import { guardClose, interceptQuit, registerExitIpc, track } from "./exit";
import { analyze, stopAnalysis } from "./analysis";
import type { AnalysisRequest } from "../shared/mix/result";
import { activeRunId, cancelRun, environment, installEngine, isRunning, startRun } from "./workers";

if (started) app.quit();

registerSchemes();

let mainWindow: BrowserWindow | null = null;
let projectDir: string | null = null;

const rendererDir = () => path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}`);

const rendererUrl = (hash = "") =>
  MAIN_WINDOW_VITE_DEV_SERVER_URL
    ? `${MAIN_WINDOW_VITE_DEV_SERVER_URL}${hash ? `#${hash}` : ""}`
    : `${APP_SCHEME}://app/index.html${hash ? `#${hash}` : ""}`;

function makeWindow(options: {
  width: number;
  height: number;
  minWidth: number;
  minHeight: number;
  x?: number;
  y?: number;
  title: string;
}): BrowserWindow {
  const win = new BrowserWindow({
    ...options,
    backgroundColor: "#101013",
    icon: app.isPackaged
      ? path.join(process.resourcesPath, "icons", "512.png")
      : path.join(app.getAppPath(), "brand", "icons", "512.png"),
    show: false,
    ...(process.platform === "darwin"
      ? { titleBarStyle: "hiddenInset" as const, trafficLightPosition: { x: 16, y: 16 } }
      : { frame: false }),
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      preload: path.join(__dirname, "preload.cjs"),
    },
  });
  win.once("ready-to-show", () => win.show());
  for (const name of [
    "maximize",
    "unmaximize",
    "enter-full-screen",
    "leave-full-screen",
    "focus",
    "blur",
  ] as const)
    win.on(name as "maximize", () => sendWindowState(win));
  win.webContents.setWindowOpenHandler(({ url }) => {
    if (/^https?:/.test(url)) void shell.openExternal(url);
    return { action: "deny" };
  });
  return win;
}

function createWindow(): void {
  mainWindow = makeWindow({
    width: 1600,
    height: 1000,
    minWidth: 1100,
    minHeight: 700,
    title: "Demoflow",
  });
  void mainWindow.loadURL(rendererUrl());
  guardClose(mainWindow);
  mainWindow.on("closed", () => {
    mainWindow = null;
    closePanelWindows();
  });
}

function assertTrustedSender(event: IpcMainInvokeEvent): void {
  const id = event.sender.id;
  if ((mainWindow && id === mainWindow.webContents.id) || isPanelSender(id)) return;
  throw new Error("Blocked IPC call from an unknown renderer.");
}

const senderWindow = (event: IpcMainInvokeEvent) => BrowserWindow.fromWebContents(event.sender);

function handle<A extends unknown[], R>(channel: string, fn: (...args: A) => Promise<R> | R): void {
  ipcMain.handle(channel, async (event, ...args) => {
    assertTrustedSender(event);
    return fn(...(args as A));
  });
}

function handleFrom<A extends unknown[], R>(
  channel: string,
  fn: (event: IpcMainInvokeEvent, ...args: A) => Promise<R> | R,
): void {
  ipcMain.handle(channel, async (event, ...args) => {
    assertTrustedSender(event);
    return fn(event, ...(args as A));
  });
}

function emitRun(event: RunEvent, total: { count: number; done: number }) {
  mainWindow?.webContents.send(IPC.runEvent, event);
  if (event.type === "run:start") {
    total.count = Math.max(1, event.nodeIds.length);
    total.done = 0;
    mainWindow?.setProgressBar(0.01);
  } else if (
    event.type === "node:done" ||
    event.type === "node:error" ||
    event.type === "node:skipped"
  ) {
    total.done += 1;
    mainWindow?.setProgressBar(Math.min(0.99, total.done / total.count));
  } else if (event.type === "run:done") {
    mainWindow?.setProgressBar(-1);
    if (Notification.isSupported() && !mainWindow?.isFocused()) {
      const count = event.outputs.length;
      notifyRunDone(
        event.ok ? "Export finished" : event.cancelled ? "Run stopped" : "Run failed",
        event.ok
          ? count === 1
            ? "Your video is ready."
            : `${count} files are ready.`
          : event.cancelled
            ? "The run was stopped before it finished."
            : (event.error ?? "Something went wrong."),
      );
    }
  }
}

function windowState(win: BrowserWindow | null): WindowState {
  return {
    maximized: win?.isMaximized() ?? false,
    fullscreen: win?.isFullScreen() ?? false,
    focused: win?.isFocused() ?? false,
  };
}

function sendWindowState(win: BrowserWindow): void {
  if (!win.isDestroyed()) win.webContents.send(IPC.windowState, windowState(win));
}

let lastNotification: Notification | null = null;

function focusMainWindow(): void {
  if (!mainWindow) return;
  if (mainWindow.isMinimized()) mainWindow.restore();
  mainWindow.show();
  mainWindow.focus();
  if (process.platform === "darwin") app.focus({ steal: true });
}

function notifyRunDone(title: string, body: string): void {
  lastNotification?.close();
  const notification = new Notification({ title, body });
  notification.on("click", () => {
    focusMainWindow();
    if (lastNotification === notification) lastNotification = null;
  });
  notification.on("close", () => {
    if (lastNotification === notification) lastNotification = null;
  });
  lastNotification = notification;
  notification.show();
}

function registerIpc(): void {
  handle(IPC.listRecent, () => listRecent());
  handle(IPC.createProject, async (opts: { name: string; template: string; dir?: string }) => {
    const p = await createProjectDir(mainWindow, opts);
    if (p) projectDir = p.dir;
    return p;
  });
  handle(IPC.openProject, async (dir?: string) => {
    const p = await openProjectDir(mainWindow, dir);
    if (p) projectDir = p.dir;
    return p;
  });
  handle(IPC.saveWorkflow, (dir: string, workflow) => saveWorkflowFile(dir, workflow as never));
  handle(IPC.readLastRun, (dir: string) => readLastRun(dir));
  handle(IPC.importAssets, (dir: string, paths?: string[]) => importAssets(mainWindow, dir, paths));
  handle(IPC.reveal, async (dir: string, rel?: string) => {
    const target = rel ? path.join(dir, rel) : dir;
    if (rel) shell.showItemInFolder(target);
    else await shell.openPath(target);
  });
  handle(IPC.run, async (request: RunRequest) => {
    if (isRunning()) throw new Error("A run is already in progress.");
    projectDir = request.projectDir;
    const runId = randomUUID();
    const total = { count: 1, done: 0 };
    await startRun(
      runId,
      request,
      () => mainWindow,
      (e) => emitRun(e, total),
      (port) =>
        (panelWindow("run") ?? mainWindow)?.webContents.postMessage(IPC.livePort, { runId }, [
          port,
        ]),
    );
    return { runId };
  });
  handle(IPC.cancel, () => cancelRun());
  handle(IPC.runStatus, () => ({
    runId: activeRunId(),
    projectDir: isRunning() ? projectDir : null,
  }));
  handle(IPC.pick, (request: PickRequest) => pickInWindow(mainWindow, request));
  handle(IPC.env, () => environment());
  handle(IPC.installEngine, () =>
    track(
      "The engine is being set up.",
      installEngine((e) => mainWindow?.webContents.send(IPC.engineEvent, e)),
    ),
  );
  handle(IPC.getSettings, () => readSettings());
  handle(IPC.setSettings, (settings: AppSettings) => writeSettings(settings));
  handle(IPC.readTrace, (dir: string, traceId: string) => readTrace(dir, traceId));
  handle(IPC.listSecrets, (dir: string) => listSecrets(dir));
  handle(IPC.listFonts, () => listLibrary());
  handle(IPC.inspectFontFiles, (paths?: string[]) => inspectFontFiles(mainWindow, paths));
  handle(IPC.inspectInstalledFonts, (faces: FontInstalledFace[]) => inspectInstalledFonts(faces));
  handle(IPC.addInspectedFonts, (token: string, families: string[]) =>
    addInspectedFonts(token, families),
  );
  handle(IPC.removeFont, (id: string) => removeLibraryFont(id));
  handle(IPC.fontCatalog, () => fontCatalog());
  handle(IPC.downloadFont, (id: string) =>
    track("A font is being downloaded.", downloadOnlineFont(id)),
  );
  handle(IPC.useFontInProject, async (dir: string, fontId: string) => {
    const def = (await listLibrary()).find((f) => f.id === fontId);
    if (!def) throw new Error("That font isn't in your library any more.");
    return copyFontToProject(dir, def);
  });
  handle(IPC.clearCache, (dir: string) => clearCache(dir));
  ipcMain.on(IPC.setTitle, (event, title: string) => {
    const id = event.sender.id;
    if ((mainWindow && id === mainWindow.webContents.id) || isPanelSender(id))
      BrowserWindow.fromWebContents(event.sender)?.setTitle(title);
  });
  handleFrom(IPC.windowRole, (event, role: WindowRole) => {
    const win = senderWindow(event);
    if (win) runWindowRole(win, role);
  });
  handleFrom(IPC.getWindowState, (event) => windowState(senderWindow(event)));
  handle(IPC.openPanelWindow, (panel: DockPanelName, dir: string) => openPanelWindow(panel, dir));
  handle(IPC.focusPanelWindow, (panel: DockPanelName) => focusPanelWindow(panel));
  handle(IPC.closePanelWindow, (panel: DockPanelName) => closePanelWindow(panel));
  handle(IPC.closePanelWindows, () => closePanelWindows());
  handle(IPC.listPanelWindows, () => openPanels());
  handle(IPC.requestStill, (request: StillRequest) => requestStill(request));
  handle(IPC.listStills, (dir: string) => listStills(dir));
  handle(IPC.analyze, (request: AnalysisRequest) => analyze(request));
}

app.whenReady().then(() => {
  registerProtocolHandlers({
    getProjectDir: () => projectDir,
    getFontsDir: fontsDir,
    rendererDir: rendererDir(),
  });
  registerIpc();
  registerExitIpc((id) => mainWindow?.webContents.id === id);
  initStills(() => mainWindow);
  initPanelWindows({
    main: () => mainWindow,
    create: (o) => makeWindow({ ...o, minWidth: 420, minHeight: 260 }),
    url: rendererUrl,
  });
  buildMenu(() => mainWindow);
  createWindow();
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

app.on("before-quit", (event) => interceptQuit(event, mainWindow));

app.on("will-quit", () => {
  if (isRunning()) cancelRun();
  stopAnalysis();
});
