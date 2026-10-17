import path from "node:path";
import { app, BrowserWindow, shell } from "electron";
import { IPC, type BroadcastPayloads, type WindowState } from "../shared/ipc";
import { guardClose } from "./exit";
import { closePanelWindows, isPanelSender, panelWindows } from "./panel-windows";
import { APP_SCHEME } from "./protocol";
import { raiseWindow } from "./raise-window";
import { closeSettingsWindow, isSettingsSender, settingsWindow } from "./settings-window";

export type WindowOptions = {
  width: number;
  height: number;
  minWidth: number;
  minHeight: number;
  x?: number;
  y?: number;
  title: string;
};

let mainWindow: BrowserWindow | null = null;

export const getMainWindow = (): BrowserWindow | null => mainWindow;

export const rendererDir = path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}`);

export function rendererUrl(hash = ""): string {
  const suffix = hash ? `#${hash}` : "";
  return MAIN_WINDOW_VITE_DEV_SERVER_URL
    ? `${MAIN_WINDOW_VITE_DEV_SERVER_URL}${suffix}`
    : `${APP_SCHEME}://app/index.html${suffix}`;
}

export function windowState(win: BrowserWindow | null): WindowState {
  return {
    maximized: win?.isMaximized() ?? false,
    fullscreen: win?.isFullScreen() ?? false,
    focused: win?.isFocused() ?? false,
  };
}

function sendWindowState(win: BrowserWindow): void {
  if (!win.isDestroyed()) win.webContents.send(IPC.windowState, windowState(win));
}

export function makeWindow(options: WindowOptions): BrowserWindow {
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
  const sendState = () => sendWindowState(win);
  win.on("maximize", sendState);
  win.on("unmaximize", sendState);
  win.on("enter-full-screen", sendState);
  win.on("leave-full-screen", sendState);
  win.on("focus", sendState);
  win.on("blur", sendState);
  win.webContents.setWindowOpenHandler(({ url }) => {
    if (/^https?:/.test(url)) void shell.openExternal(url);
    return { action: "deny" };
  });
  return win;
}

export function createMainWindow(): void {
  const win = makeWindow({
    width: 1600,
    height: 1000,
    minWidth: 1100,
    minHeight: 700,
    title: "Demoflow",
  });
  mainWindow = win;
  void win.loadURL(rendererUrl());
  guardClose(win);
  win.on("closed", () => {
    mainWindow = null;
    closePanelWindows();
    closeSettingsWindow();
  });
}

export function focusMainWindow(): void {
  if (!mainWindow) return;
  raiseWindow(mainWindow);
  if (process.platform === "darwin") app.focus({ steal: true });
}

export const isEditorSender = (id: number): boolean => mainWindow?.webContents.id === id;

export const isTrusted = (id: number): boolean =>
  isEditorSender(id) || isPanelSender(id) || isSettingsSender(id);

function appWindows(): BrowserWindow[] {
  return [mainWindow, ...panelWindows(), settingsWindow()].filter(
    (w): w is BrowserWindow => !!w && !w.isDestroyed(),
  );
}

export function broadcast<C extends keyof BroadcastPayloads>(
  channel: C,
  payload: BroadcastPayloads[C],
): void {
  for (const win of appWindows()) win.webContents.send(channel, payload);
}
