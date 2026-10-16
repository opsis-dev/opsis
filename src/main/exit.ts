import { app, ipcMain, type BrowserWindow } from "electron";
import { IPC, type CloseDecision, type CloseRequest } from "../shared/ipc";
import { readSettings, writeSettings } from "./settings";
import { cancelRun, isRunning } from "./workers";

const tasks = new Map<string, number>();
let unsaved = false;
let allowClose = false;
let quitting = false;

export async function track<T>(label: string, work: Promise<T>): Promise<T> {
  tasks.set(label, (tasks.get(label) ?? 0) + 1);
  try {
    return await work;
  } finally {
    const left = (tasks.get(label) ?? 1) - 1;
    if (left > 0) tasks.set(label, left);
    else tasks.delete(label);
  }
}

export function setUnsaved(value: boolean): void {
  unsaved = value;
}

function ongoing(): string[] {
  return [
    ...(isRunning() ? ["A run is in progress. Recording and exporting will stop."] : []),
    ...tasks.keys(),
    ...(unsaved ? ["Your latest changes are still being saved."] : []),
  ];
}

function flushSave(win: BrowserWindow): Promise<void> {
  if (!unsaved || win.isDestroyed()) return Promise.resolve();
  return new Promise((resolve) => {
    const done = () => {
      clearTimeout(timer);
      ipcMain.removeListener(IPC.saveFlushed, done);
      resolve();
    };
    const timer = setTimeout(done, 3000);
    ipcMain.once(IPC.saveFlushed, done);
    win.webContents.send(IPC.flushSave);
  });
}

const SHOWN_WITHIN_MS = 2000;

let decide: ((decision: CloseDecision) => void) | null = null;
let shown: (() => void) | null = null;

function askRenderer(win: BrowserWindow, request: CloseRequest): Promise<CloseDecision> {
  if (win.isDestroyed() || win.webContents.isCrashed())
    return Promise.resolve({ close: true, dontAsk: false });
  return new Promise((resolve) => {
    const unanswered = setTimeout(() => decide?.({ close: true, dontAsk: false }), SHOWN_WITHIN_MS);
    shown = () => {
      shown = null;
      clearTimeout(unanswered);
    };
    decide = (decision) => {
      decide = null;
      shown = null;
      clearTimeout(unanswered);
      resolve(decision);
    };
    win.webContents.send(IPC.confirmClose, request);
  });
}

async function confirmClose(win: BrowserWindow): Promise<boolean> {
  const settings = await readSettings();
  const busy = ongoing();
  const ask = settings.app.confirmExit;
  if (!ask && busy.length === 0) return true;
  if (win.isMinimized()) win.restore();
  win.show();
  win.focus();
  const decision = await askRenderer(win, { ask, busy });
  if (!decision.close) return false;
  if (ask && decision.dontAsk)
    await writeSettings({ ...settings, app: { ...settings.app, confirmExit: false } });
  await flushSave(win);
  if (isRunning()) cancelRun();
  return true;
}

export function guardClose(win: BrowserWindow): void {
  let asking = false;
  win.on("close", (event) => {
    if (allowClose) return;
    event.preventDefault();
    if (asking) return;
    asking = true;
    void confirmClose(win)
      .then((ok) => {
        if (!ok) {
          quitting = false;
          return;
        }
        allowClose = true;
        if (quitting) app.quit();
        else win.close();
      })
      .finally(() => {
        asking = false;
      });
  });
  win.on("closed", () => {
    allowClose = false;
  });
}

export function interceptQuit(
  event: { preventDefault: () => void },
  win: BrowserWindow | null,
): void {
  if (allowClose || !win || win.isDestroyed()) return;
  event.preventDefault();
  quitting = true;
  win.close();
}

export function registerExitIpc(isTrusted: (id: number) => boolean): void {
  ipcMain.on(IPC.reportUnsaved, (event, value: boolean) => {
    if (isTrusted(event.sender.id)) setUnsaved(!!value);
  });
  ipcMain.on(IPC.closeShown, (event) => {
    if (isTrusted(event.sender.id)) shown?.();
  });
  ipcMain.on(IPC.closeDecision, (event, decision: CloseDecision) => {
    if (isTrusted(event.sender.id))
      decide?.({ close: !!decision?.close, dontAsk: !!decision?.dontAsk });
  });
}
