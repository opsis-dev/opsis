import { BrowserWindow, session as electronSession } from "electron";
import { z } from "zod";
import { parseJsonWith } from "../shared/guards";
import { PICKED_BY, type PickRequest, type PickResult } from "../shared/ipc";
import type { LoginRequest } from "../shared/worker-protocol";
import { storageState, type LocalStorageByOrigin } from "./login-state";
import { pickerScript } from "./picker-script";

const PICK_PREFIX = "__demoflow_pick__:";
const PICKER_SCRIPT = pickerScript(PICK_PREFIX);
const SELECTOR_POLL_MS = 500;

const pickResultSchema = z.object({
  target: z.string(),
  css: z.string(),
  role: z.string().optional(),
  selectorUsed: z.enum(PICKED_BY),
});

function sandboxedWindow(
  parent: BrowserWindow | null,
  title: string,
  partition: string,
  size?: { width: number; height: number },
) {
  return new BrowserWindow({
    parent: parent ?? undefined,
    width: size?.width ?? 1280,
    height: size?.height ?? 860,
    useContentSize: !!size,
    title,
    autoHideMenuBar: true,
    webPreferences: { partition, sandbox: true, contextIsolation: true, nodeIntegration: false },
  });
}

function parsePick(message: string): PickResult | undefined {
  return parseJsonWith(pickResultSchema, message.slice(PICK_PREFIX.length))?.data;
}

export function pickInWindow(parent: BrowserWindow | null, req: PickRequest): Promise<PickResult> {
  return new Promise((resolve) => {
    const win = sandboxedWindow(
      parent,
      "Pick an element — click it, or Shift-click a spot",
      `demoflow-picker-${Date.now()}`,
      req.viewport,
    );
    let result: PickResult = null;
    const inject = () =>
      void win.webContents.executeJavaScript(PICKER_SCRIPT).catch(() => {
        if (!win.isDestroyed() && !win.webContents.isLoading()) win.close();
      });
    win.webContents.on("did-finish-load", inject);
    win.webContents.on("console-message", (event) => {
      if (!event.message.startsWith(PICK_PREFIX)) return;
      const picked = parsePick(event.message);
      if (!picked) return;
      result = picked;
      win.close();
    });
    win.on("closed", () => resolve(result));
    void win.loadURL(req.url);
  });
}

async function readLocalStorage(win: BrowserWindow, origins: LocalStorageByOrigin): Promise<void> {
  try {
    const url = new URL(win.webContents.getURL());
    if (!/^https?:$/.test(url.protocol)) return;
    const entries: Array<[string, string]> = await win.webContents.executeJavaScript(
      "Object.entries(localStorage)",
    );
    origins.set(
      url.origin,
      entries.map(([name, value]) => ({ name, value })),
    );
  } catch {
    return;
  }
}

const selectorShown = (win: BrowserWindow, selector: string): Promise<boolean> =>
  win.webContents
    .executeJavaScript(`!!document.querySelector(${JSON.stringify(selector)})`)
    .catch(() => false);

function watchForLogin(
  win: BrowserWindow,
  req: LoginRequest,
  on: { page: () => void; done: () => void },
): () => void {
  let stopped = false;
  let landedUrl: string | null = null;
  const timer = setTimeout(on.done, req.timeoutSec * 1000);
  const check = () => {
    if (stopped || win.isDestroyed()) return;
    on.page();
    const url = win.webContents.getURL();
    if (!url || url === "about:blank") return;
    if (landedUrl === null) landedUrl = url;
    else if (req.doneWhen === "url-change" && url !== landedUrl) on.done();
    if (req.doneWhen === "selector" && req.selector)
      void selectorShown(win, req.selector).then((found) => {
        if (found) on.done();
      });
  };
  const poll = req.doneWhen === "selector" ? setInterval(check, SELECTOR_POLL_MS) : undefined;
  win.webContents.on("did-finish-load", check);
  win.webContents.on("did-navigate-in-page", check);
  return () => {
    stopped = true;
    clearTimeout(timer);
    clearInterval(poll);
  };
}

export function loginInWindow(
  parent: BrowserWindow | null,
  req: LoginRequest,
): Promise<string | null> {
  return new Promise((resolve) => {
    const partition = `demoflow-login-${Date.now()}`;
    const win = sandboxedWindow(parent, "Log in — close this window when you're done", partition);
    const origins: LocalStorageByOrigin = new Map();
    let finished = false;
    const stopWatching = watchForLogin(win, req, {
      page: () => void readLocalStorage(win, origins),
      done: () => void finish(),
    });
    const finish = async () => {
      if (finished) return;
      finished = true;
      stopWatching();
      await readLocalStorage(win, origins);
      const cookies = await electronSession.fromPartition(partition).cookies.get({});
      if (!win.isDestroyed()) win.destroy();
      resolve(JSON.stringify(storageState(cookies, origins)));
    };
    win.on("close", (event) => {
      if (finished) return;
      event.preventDefault();
      void finish();
    });
    void win.loadURL(req.url);
  });
}
