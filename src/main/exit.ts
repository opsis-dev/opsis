import { app, ipcMain, type BrowserWindow } from "electron";
import { IPC, type CloseDecision, type CloseRequest } from "../shared/ipc";
import { raiseWindow } from "./raise-window";
import { cancelRun, isRunning } from "./run-host";
import { patchSettings, readSettings } from "./settings";
import { isEditorSender } from "./window-registry";

const SHOWN_WITHIN_MS = 2000;
const FLUSH_WITHIN_MS = 3000;
const UNLOAD_WITHIN_MS = 3000;

type CloseQuestion = { shown: () => void; decide: (decision: CloseDecision) => void };

const tasks = new Map<string, number>();
let unsaved = false;
let allowClose = false;
let quitting = false;
let question: CloseQuestion | null = null;

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
    const timer = setTimeout(done, FLUSH_WITHIN_MS);
    ipcMain.once(IPC.saveFlushed, done);
    win.webContents.send(IPC.flushSave);
  });
}

function askRenderer(win: BrowserWindow, request: CloseRequest): Promise<CloseDecision> {
  if (win.isDestroyed() || win.webContents.isCrashed())
    return Promise.resolve({ close: true, dontAsk: false });
  return new Promise((resolve) => {
    const asked: CloseQuestion = {
      shown: () => clearTimeout(unanswered),
      decide: (decision) => {
        clearTimeout(unanswered);
        if (question === asked) question = null;
        resolve(decision);
      },
    };
    const unanswered = setTimeout(
      () => asked.decide({ close: true, dontAsk: false }),
      SHOWN_WITHIN_MS,
    );
    question = asked;
    win.webContents.send(IPC.confirmClose, request);
  });
}

async function confirmClose(win: BrowserWindow): Promise<boolean> {
  const ask = readSettings().app.confirmExit;
  const busy = ongoing();
  if (!ask && busy.length === 0) return true;
  raiseWindow(win);
  const decision = await askRenderer(win, { ask, busy });
  if (!decision.close) return false;
  if (ask && decision.dontAsk) patchSettings({ app: { confirmExit: false } });
  await flushSave(win);
  cancelRun();
  return true;
}

function closeEvenIfPageDies(win: BrowserWindow): void {
  const force = () => {
    clearTimeout(unanswered);
    if (win.isDestroyed()) return;
    win.destroy();
    if (quitting) app.quit();
  };
  const unanswered = setTimeout(force, UNLOAD_WITHIN_MS);
  win.once("closed", () => clearTimeout(unanswered));
  win.webContents.once("render-process-gone", force);
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
        closeEvenIfPageDies(win);
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

export function interceptQuit(event: Electron.Event, win: BrowserWindow | null): void {
  if (allowClose || !win || win.isDestroyed()) return;
  event.preventDefault();
  quitting = true;
  win.close();
}

export function registerExitIpc(): void {
  ipcMain.on(IPC.reportUnsaved, (event, value: boolean) => {
    if (isEditorSender(event.sender.id)) unsaved = value;
  });
  ipcMain.on(IPC.closeShown, (event) => {
    if (isEditorSender(event.sender.id)) question?.shown();
  });
  ipcMain.on(IPC.closeDecision, (event, decision: CloseDecision) => {
    if (isEditorSender(event.sender.id)) question?.decide(decision);
  });
}
