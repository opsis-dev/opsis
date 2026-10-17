import type { BrowserWindow, Rectangle } from "electron";
import type { SettingsPage } from "../shared/ipc";
import {
  isVisible,
  loadBounds,
  rectangleSchema,
  rememberBounds,
  saveBounds,
} from "./window-bounds";

type SettingsWindowHost = {
  create: (options: {
    width: number;
    height: number;
    minWidth: number;
    minHeight: number;
    x?: number;
    y?: number;
    title: string;
  }) => BrowserWindow;
  url: (hash: string) => string;
};

const BOUNDS_FILE = "settings-window.json";

let host: SettingsWindowHost | null = null;
let win: BrowserWindow | null = null;

async function savedBounds(): Promise<Rectangle | undefined> {
  const rect = await loadBounds(BOUNDS_FILE, rectangleSchema);
  return rect && isVisible(rect) ? rect : undefined;
}

export function initSettingsWindow(next: SettingsWindowHost): void {
  host = next;
}

export const settingsWindow = () => (win && !win.isDestroyed() ? win : null);

export const isSettingsSender = (id: number) => settingsWindow()?.webContents.id === id;

export async function openSettingsWindow(page?: SettingsPage): Promise<void> {
  if (!host) return;
  const hash = `/settings${page ? `?page=${page}` : ""}`;
  const open = settingsWindow();
  if (open) {
    if (page) void open.loadURL(host.url(hash));
    if (open.isMinimized()) open.restore();
    open.show();
    open.focus();
    return;
  }
  const created = host.create({
    width: 980,
    height: 720,
    minWidth: 720,
    minHeight: 480,
    ...(await savedBounds()),
    title: "Settings — Demoflow",
  });
  win = created;
  rememberBounds(created, (rect) => saveBounds(BOUNDS_FILE, rect));
  created.on("closed", () => {
    if (win === created) win = null;
  });
  void created.loadURL(host.url(hash));
}

export function closeSettingsWindow(): void {
  settingsWindow()?.close();
}
