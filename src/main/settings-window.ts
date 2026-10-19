import type { BrowserWindow } from "electron";
import { IPC, type SettingsPage } from "../shared/ipc";
import { openRememberedWindow } from "./window-bounds";
import { liveWindow } from "./window-registry";

let win: BrowserWindow | null = null;

export const settingsWindow = () => liveWindow(win);

export const isSettingsSender = (id: number) => settingsWindow()?.webContents.id === id;

export async function openSettingsWindow(page?: SettingsPage): Promise<void> {
  const created = await openRememberedWindow({
    key: "settings",
    existing: settingsWindow(),
    options: {
      width: 980,
      height: 720,
      minWidth: 720,
      minHeight: 480,
      title: "Settings — Opsis",
    },
    hash: `/settings${page ? `?page=${page}` : ""}`,
    onClosed: (closed) => {
      if (win === closed) win = null;
    },
  });
  if (created) win = created;
  else if (page) settingsWindow()?.webContents.send(IPC.settingsPage, page);
}

export function closeSettingsWindow(): void {
  settingsWindow()?.close();
}
