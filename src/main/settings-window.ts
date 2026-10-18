import type { BrowserWindow, Rectangle } from "electron";
import { IPC, type SettingsPage } from "../shared/ipc";
import { boundsSchema } from "../shared/schema/actions";
import { boundsStore, openRememberedWindow } from "./window-bounds";
import { liveWindow } from "./window-registry";

const store = boundsStore<Rectangle>("settings-window.json", boundsSchema);

let win: BrowserWindow | null = null;

export const settingsWindow = () => liveWindow(win);

export const isSettingsSender = (id: number) => settingsWindow()?.webContents.id === id;

export async function openSettingsWindow(page?: SettingsPage): Promise<void> {
  const created = await openRememberedWindow({
    key: "settings",
    existing: settingsWindow(),
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
  else if (page) settingsWindow()?.webContents.send(IPC.settingsPage, page);
}

export function closeSettingsWindow(): void {
  settingsWindow()?.close();
}
