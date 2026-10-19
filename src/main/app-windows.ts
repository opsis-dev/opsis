import type { BrowserWindow } from "electron";
import type { BroadcastPayloads } from "../shared/ipc";
import { guardClose } from "./exit";
import { closePanelWindows, isPanelSender, panelWindows } from "./panel-windows";
import { closeSettingsWindow, isSettingsSender, settingsWindow } from "./settings-window";
import {
  getMainWindow,
  isEditorSender,
  liveWindow,
  makeWindow,
  rendererUrl,
  setMainWindow,
} from "./window-registry";

export function createMainWindow(): void {
  const win = makeWindow({
    width: 1600,
    height: 1000,
    minWidth: 1100,
    minHeight: 700,
    title: "Opsis",
  });
  setMainWindow(win);
  void win.loadURL(rendererUrl());
  guardClose(win);
  win.on("closed", () => {
    setMainWindow(null);
    closePanelWindows();
    closeSettingsWindow();
  });
}

export const isTrusted = (id: number): boolean =>
  isEditorSender(id) || isPanelSender(id) || isSettingsSender(id);

function appWindows(): BrowserWindow[] {
  return [liveWindow(getMainWindow()), ...panelWindows(), settingsWindow()].filter(
    (w) => w !== null,
  );
}

export function broadcast<C extends keyof BroadcastPayloads>(
  channel: C,
  payload: BroadcastPayloads[C],
): void {
  for (const win of appWindows()) win.webContents.send(channel, payload);
}
