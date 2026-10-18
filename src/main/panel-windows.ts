import type { BrowserWindow, Rectangle } from "electron";
import { z } from "zod";
import { DOCK_PANELS, DOCK_TITLES, IPC, type DockPanelName } from "../shared/ipc";
import { raiseWindow } from "./raise-window";
import { boundsStore, openRememberedWindow, rectangleSchema } from "./window-bounds";
import { getMainWindow, liveWindow } from "./window-registry";

const MIN_SIZE = { minWidth: 420, minHeight: 260 };

const DEFAULT_SIZE: Record<DockPanelName, { width: number; height: number }> = {
  preview: { width: 960, height: 640 },
  timeline: { width: 1400, height: 460 },
  mixer: { width: 1100, height: 560 },
  run: { width: 760, height: 560 },
};

type PanelBounds = Partial<Record<DockPanelName, Rectangle>>;

const store = boundsStore<PanelBounds>(
  "panel-windows.json",
  z.partialRecord(z.enum(DOCK_PANELS), rectangleSchema),
);

const windows = new Map<DockPanelName, BrowserWindow>();
let bounds: PanelBounds | null = null;

async function panelBounds(): Promise<PanelBounds> {
  bounds ??= (await store.load()) ?? {};
  return bounds;
}

function savePanelBounds(panel: DockPanelName, rect: Rectangle): void {
  bounds = { ...bounds, [panel]: rect };
  store.save(bounds);
}

export const openPanels = (): DockPanelName[] => [...windows.keys()];

function notify(): void {
  getMainWindow()?.webContents.send(IPC.panelWindows, openPanels());
}

export const panelWindow = (panel: DockPanelName) => liveWindow(windows.get(panel));

export const panelWindows = (): BrowserWindow[] =>
  [...windows.values()].filter((w) => !w.isDestroyed());

export const isPanelSender = (id: number) => panelWindows().some((w) => w.webContents.id === id);

export async function openPanelWindow(panel: DockPanelName, projectDir: string): Promise<void> {
  const win = await openRememberedWindow({
    existing: panelWindow(panel),
    saved: async () => (await panelBounds())[panel],
    save: (rect) => savePanelBounds(panel, rect),
    options: { ...DEFAULT_SIZE[panel], ...MIN_SIZE, title: `${DOCK_TITLES[panel]} — Demoflow` },
    hash: `/panel/${panel}?project=${encodeURIComponent(projectDir)}`,
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
