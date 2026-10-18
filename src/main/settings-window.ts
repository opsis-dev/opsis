import type { BrowserWindow, Rectangle } from "electron";
import { IPC, type SettingsPage } from "../shared/ipc";
import { boundsStore, openRememberedWindow, rectangleSchema } from "./window-bounds";
import { liveWindow } from "./window-registry";

const store = boundsStore<Rectangle>("settings-window.json", rectangleSchema);

let win: BrowserWindow | null = null;

export const settingsWindow = () => liveWindow(win);

export const isSettingsSender = (id: number) => settingsWindow()?.webContents.id === id;

export async function openSettingsWindow(page?: SettingsPage): Promise<void> {
  const open = settingsWindow();
  if (open && page) open.webContents.send(IPC.settingsPage, page);
  const created = await openRememberedWindow({
    existing: open,
    saved: store.load,
    save: store.save,
    options: {
      width: 980,
      height: 720,
      minWidth: 720,
      minHeight: 480,
      title: "Settings — Demoflow",
    },
    hash: `/settings${page ? `?page=${page}` : ""}`,
    onClosed: (closed) => {
      if (win === closed) win = null;
    },
  });
  if (created) win = created;
}

export function closeSettingsWindow(): void {
  settingsWindow()?.close();
}
