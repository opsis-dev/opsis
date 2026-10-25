import { DOCK_TITLES, IPC, type DockPanelName } from "@opsis/core/ipc";
import type { BrowserWindow } from "electron";
import { raiseWindow } from "./raise-window";
import { openRememberedWindow } from "./window-bounds";
import { getMainWindow, liveWindow } from "./window-registry";

const MIN_SIZE = { minWidth: 420, minHeight: 260 };

const DEFAULT_SIZE: Record<DockPanelName, { width: number; height: number }> = {
  preview: { width: 960, height: 640 },
  timeline: { width: 1400, height: 460 },
  mixer: { width: 1100, height: 560 },
  run: { width: 760, height: 560 },
};

const windows = new Map<DockPanelName, BrowserWindow>();

export const openPanels = (): DockPanelName[] => [...windows.keys()];

function notify(): void {
  getMainWindow()?.webContents.send(IPC.panelWindows, openPanels());
}

export const panelWindow = (panel: DockPanelName) => liveWindow(windows.get(panel));

export const panelWindows = (): BrowserWindow[] =>
  [...windows.values()].map(liveWindow).filter((w) => w !== null);

export const isPanelSender = (id: number) => panelWindows().some((w) => w.webContents.id === id);

export async function openPanelWindow(panel: DockPanelName): Promise<void> {
  const win = await openRememberedWindow({
    key: `panel:${panel}`,
    existing: panelWindow(panel),
    options: { ...DEFAULT_SIZE[panel], ...MIN_SIZE, title: `${DOCK_TITLES[panel]} — Opsis` },
    hash: `/panel/${panel}`,
    onClosed: (closed) => {
      if (windows.get(panel) === closed) windows.delete(panel);
      notify();
    },
  });
  if (!win) return;
  windows.set(panel, win);
  notify();
}

export function focusPanelWindow(panel: DockPanelName): void {
  const win = panelWindow(panel);
  if (win) raiseWindow(win);
}

export function closePanelWindow(panel: DockPanelName): void {
  panelWindow(panel)?.close();
}

export function closePanelWindows(): void {
  for (const win of panelWindows()) win.close();
}
