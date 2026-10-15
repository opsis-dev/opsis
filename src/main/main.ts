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
import { buildMenu } from "./menu";
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
    backgroundColor: "#09090b",
    title: "Demoflow",
    show: false,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      preload: path.join(__dirname, "preload.cjs"),
    },
  });
  mainWindow.once("ready-to-show", () => mainWindow?.show());
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
      new Notification({
        title: event.ok ? "Render finished" : event.cancelled ? "Run cancelled" : "Run failed",
        body: event.ok ? `${event.outputs.length} file(s) in .demoflow/out` : (event.error ?? ""),
      }).show();
    }
  }
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
  handle(IPC.clearCache, (dir: string) => clearCache(dir));
  ipcMain.on(IPC.setTitle, (event, title: string) => {
    if (mainWindow && event.sender.id === mainWindow.webContents.id) mainWindow.setTitle(title);
  });
}

app.whenReady().then(() => {
  registerProtocolHandlers({ getProjectDir: () => projectDir, rendererDir: rendererDir() });
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
