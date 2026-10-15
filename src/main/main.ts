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
import type { FontInstalledFace, WindowState } from "../shared/ipc";
import { activeRunId, cancelRun, environment, installEngine, isRunning, startRun } from "./workers";

if (started) app.quit();

registerSchemes();

let mainWindow: BrowserWindow | null = null;
let projectDir: string | null = null;

const rendererDir = () => path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}`);

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1600,
    height: 1000,
    minWidth: 1100,
    minHeight: 700,
    backgroundColor: "#101013",
    icon: app.isPackaged
      ? path.join(process.resourcesPath, "icons", "512.png")
      : path.join(app.getAppPath(), "brand", "icons", "512.png"),
    title: "Demoflow",
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
  mainWindow.once("ready-to-show", () => mainWindow?.show());
  for (const name of [
    "maximize",
    "unmaximize",
    "enter-full-screen",
    "leave-full-screen",
    "focus",
    "blur",
  ] as const)
    mainWindow.on(name as "maximize", () => sendWindowState());
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (/^https?:/.test(url)) void shell.openExternal(url);
    return { action: "deny" };
  });
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) void mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
  else void mainWindow.loadURL(`${APP_SCHEME}://app/index.html`);
  mainWindow.on("closed", () => {
    mainWindow = null;
  });
}

function assertTrustedSender(event: IpcMainInvokeEvent): void {
  if (!mainWindow || event.sender.id !== mainWindow.webContents.id)
    throw new Error("Blocked IPC call from an unknown renderer.");
}

function handle<A extends unknown[], R>(channel: string, fn: (...args: A) => Promise<R> | R): void {
  ipcMain.handle(channel, async (event, ...args) => {
    assertTrustedSender(event);
    return fn(...(args as A));
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

function windowState(): WindowState {
  return {
    maximized: mainWindow?.isMaximized() ?? false,
    fullscreen: mainWindow?.isFullScreen() ?? false,
    focused: mainWindow?.isFocused() ?? false,
  };
}

function sendWindowState(): void {
  if (mainWindow && !mainWindow.isDestroyed())
    mainWindow.webContents.send(IPC.windowState, windowState());
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
      (port) => mainWindow?.webContents.postMessage(IPC.livePort, { runId }, [port]),
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
    installEngine((e) => mainWindow?.webContents.send(IPC.engineEvent, e)),
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
  handle(IPC.downloadFont, (id: string) => downloadOnlineFont(id));
  handle(IPC.useFontInProject, async (dir: string, fontId: string) => {
    const def = (await listLibrary()).find((f) => f.id === fontId);
    if (!def) throw new Error("That font isn't in your library any more.");
    return copyFontToProject(dir, def);
  });
  handle(IPC.clearCache, (dir: string) => clearCache(dir));
  ipcMain.on(IPC.setTitle, (event, title: string) => {
    if (mainWindow && event.sender.id === mainWindow.webContents.id) mainWindow.setTitle(title);
  });
  handle(IPC.windowRole, (role: WindowRole) => {
    if (mainWindow) runWindowRole(mainWindow, role);
  });
  handle(IPC.getWindowState, () => windowState());
}

app.whenReady().then(() => {
  registerProtocolHandlers({
    getProjectDir: () => projectDir,
    getFontsDir: fontsDir,
    rendererDir: rendererDir(),
  });
  registerIpc();
  buildMenu(() => mainWindow);
  createWindow();
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

app.on("before-quit", () => {
  if (isRunning()) cancelRun();
});
