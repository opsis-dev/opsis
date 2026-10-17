import type { BrowserWindow, Rectangle } from "electron";
import type { SettingsPage } from "../shared/ipc";
import type { WindowOptions } from "./app-windows";
import { raiseWindow } from "./raise-window";
import { boundsStore, isVisible, rectangleSchema, rememberBounds } from "./window-bounds";

type SettingsWindowHost = {
  create: (options: WindowOptions) => BrowserWindow;
  url: (hash: string) => string;
};

const store = boundsStore<Rectangle>("settings-window.json", rectangleSchema);

let host: SettingsWindowHost | null = null;
let win: BrowserWindow | null = null;

async function savedBounds(): Promise<Rectangle | undefined> {
  const rect = await store.load();
  return rect && isVisible(rect) ? rect : undefined;
}

export function initSettingsWindow(next: SettingsWindowHost): void {
  host = next;
}

function settingsHost(): SettingsWindowHost {
  if (!host) throw new Error("The Settings window was used before the app finished starting.");
  return host;
}

export const settingsWindow = () => (win && !win.isDestroyed() ? win : null);

export const isSettingsSender = (id: number) => settingsWindow()?.webContents.id === id;

export async function openSettingsWindow(page?: SettingsPage): Promise<void> {
  const { create, url } = settingsHost();
  const hash = `/settings${page ? `?page=${page}` : ""}`;
  const open = settingsWindow();
  if (open) {
    if (page) void open.loadURL(url(hash));
    raiseWindow(open);
    return;
  }
  const created = create({
    width: 980,
    height: 720,
    minWidth: 720,
    minHeight: 480,
    ...(await savedBounds()),
    title: "Settings — Demoflow",
  });
  win = created;
  rememberBounds(created, store.save);
  created.on("closed", () => {
    if (win === created) win = null;
  });
  void created.loadURL(url(hash));
}

export function closeSettingsWindow(): void {
  settingsWindow()?.close();
}
