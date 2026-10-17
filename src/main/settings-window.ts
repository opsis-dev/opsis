import type { BrowserWindow, Rectangle } from "electron";
import type { SettingsPage } from "../shared/ipc";
import type { WindowOptions } from "./app-windows";
import { raiseWindow } from "./raise-window";
import {
  boundsStore,
  liveWindow,
  rectangleSchema,
  rememberBounds,
  requireHost,
  visibleBounds,
} from "./window-bounds";

type SettingsWindowHost = {
  create: (options: WindowOptions) => BrowserWindow;
  url: (hash: string) => string;
};

const store = boundsStore<Rectangle>("settings-window.json", rectangleSchema);

let host: SettingsWindowHost | null = null;
let win: BrowserWindow | null = null;

export function initSettingsWindow(next: SettingsWindowHost): void {
  host = next;
}

export const settingsWindow = () => liveWindow(win);

export const isSettingsSender = (id: number) => settingsWindow()?.webContents.id === id;

export async function openSettingsWindow(page?: SettingsPage): Promise<void> {
  const { create, url } = requireHost(host, "The Settings window was");
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
    ...visibleBounds(await store.load()),
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
