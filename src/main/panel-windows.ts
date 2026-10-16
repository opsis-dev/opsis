import type { BrowserWindow, Rectangle } from "electron";
import { IPC, type DockPanelName } from "../shared/ipc";
import { isVisible, loadBounds, rememberBounds, saveBounds } from "./window-bounds";

export type PanelWindowHost = {
  main: () => BrowserWindow | null;
  create: (options: {
    width: number;
    height: number;
    x?: number;
    y?: number;
    title: string;
  }) => BrowserWindow;
  url: (hash: string) => string;
};

const TITLES: Record<DockPanelName, string> = {
  preview: "Preview",
  timeline: "Timeline",
  mixer: "Mixer",
  run: "Run",
};

const DEFAULT_SIZE: Record<DockPanelName, { width: number; height: number }> = {
  preview: { width: 960, height: 640 },
  timeline: { width: 1400, height: 460 },
  mixer: { width: 1100, height: 560 },
  run: { width: 760, height: 560 },
};

const windows = new Map<DockPanelName, BrowserWindow>();
let host: PanelWindowHost | null = null;
let bounds: Partial<Record<DockPanelName, Rectangle>> | null = null;

const BOUNDS_FILE = "panel-windows.json";

async function panelBounds(): Promise<Partial<Record<DockPanelName, Rectangle>>> {
  bounds ??= (await loadBounds<Partial<Record<DockPanelName, Rectangle>>>(BOUNDS_FILE)) ?? {};
  return bounds;
}

function savePanelBounds(panel: DockPanelName, rect: Rectangle): void {
  bounds = { ...bounds, [panel]: rect };
  saveBounds(BOUNDS_FILE, bounds);
}

export function initPanelWindows(next: PanelWindowHost): void {
  host = next;
}

export const openPanels = (): DockPanelName[] => [...windows.keys()];

function notify(): void {
  host?.main()?.webContents.send(IPC.panelWindows, openPanels());
}

export async function openPanelWindow(panel: DockPanelName, projectDir: string): Promise<void> {
  if (!host) return;
  const existing = windows.get(panel);
  if (existing && !existing.isDestroyed()) {
    existing.show();
    existing.focus();
    return;
  }
  const saved = (await panelBounds())[panel];
  const size = DEFAULT_SIZE[panel];
  const rect = saved && isVisible(saved) ? saved : undefined;
  const win = host.create({ ...size, ...rect, title: `${TITLES[panel]} — Demoflow` });
  windows.set(panel, win);
  rememberBounds(win, (next) => savePanelBounds(panel, next));
  win.on("closed", () => {
    if (windows.get(panel) === win) windows.delete(panel);
    notify();
  });
  void win.loadURL(host.url(`/panel/${panel}?project=${encodeURIComponent(projectDir)}`));
  notify();
}

export function focusPanelWindow(panel: DockPanelName): void {
  const win = windows.get(panel);
  if (!win || win.isDestroyed()) return;
  if (win.isMinimized()) win.restore();
  win.show();
  win.focus();
}

export function closePanelWindow(panel: DockPanelName): void {
  windows.get(panel)?.close();
}

export function closePanelWindows(): void {
  for (const win of windows.values()) if (!win.isDestroyed()) win.close();
}

export const panelWindow = (panel: DockPanelName) => {
  const win = windows.get(panel);
  return win && !win.isDestroyed() ? win : null;
};

export const panelWindows = (): BrowserWindow[] =>
  [...windows.values()].filter((w) => !w.isDestroyed());

export const isPanelSender = (id: number) =>
  [...windows.values()].some((w) => !w.isDestroyed() && w.webContents.id === id);
