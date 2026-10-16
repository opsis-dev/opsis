import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { app, screen, type BrowserWindow, type Rectangle } from "electron";
import type { SettingsPage } from "../shared/ipc";

export type SettingsWindowHost = {
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

let host: SettingsWindowHost | null = null;
let win: BrowserWindow | null = null;

const boundsFile = () => path.join(app.getPath("userData"), "settings-window.json");

async function savedBounds(): Promise<Rectangle | undefined> {
  try {
    const rect = JSON.parse(await readFile(boundsFile(), "utf8")) as Rectangle;
    const shown = screen.getAllDisplays().some((d) => {
      const a = d.workArea;
      return rect.x < a.x + a.width - 40 && rect.x + rect.width > a.x + 40 && rect.y >= a.y - 20;
    });
    return shown ? rect : undefined;
  } catch {
    return undefined;
  }
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
  const remember = () => {
    if (!created.isDestroyed() && !created.isMinimized() && !created.isMaximized())
      void writeFile(boundsFile(), JSON.stringify(created.getBounds())).catch(() => undefined);
  };
  created.on("resized", remember);
  created.on("moved", remember);
  created.on("close", remember);
  created.on("closed", () => {
    if (win === created) win = null;
  });
  void created.loadURL(host.url(hash));
}

export function closeSettingsWindow(): void {
  settingsWindow()?.close();
}
