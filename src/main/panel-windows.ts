import type { BrowserWindow, Rectangle } from "electron";
import { z } from "zod";
import { DOCK_PANELS, IPC, type DockPanelName } from "../shared/ipc";
import type { WindowOptions } from "./app-windows";
import { raiseWindow } from "./raise-window";
import { boundsStore, isVisible, rectangleSchema, rememberBounds } from "./window-bounds";

type PanelWindowHost = {
  main: () => BrowserWindow | null;
  create: (options: Omit<WindowOptions, "minWidth" | "minHeight">) => BrowserWindow;
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

type PanelBounds = Partial<Record<DockPanelName, Rectangle>>;

const store = boundsStore<PanelBounds>(
  "panel-windows.json",
  z.partialRecord(z.enum(DOCK_PANELS), rectangleSchema),
);

const windows = new Map<DockPanelName, BrowserWindow>();
let host: PanelWindowHost | null = null;
let bounds: PanelBounds | null = null;

async function panelBounds(): Promise<PanelBounds> {
  bounds ??= (await store.load()) ?? {};
  return bounds;
}

function savePanelBounds(panel: DockPanelName, rect: Rectangle): void {
  bounds = { ...bounds, [panel]: rect };
  store.save(bounds);
}

export function initPanelWindows(next: PanelWindowHost): void {
  host = next;
}

function panelHost(): PanelWindowHost {
  if (!host) throw new Error("Panel windows were used before the app finished starting.");
  return host;
}

export const openPanels = (): DockPanelName[] => [...windows.keys()];

function notify(): void {
  host?.main()?.webContents.send(IPC.panelWindows, openPanels());
}

export const panelWindow = (panel: DockPanelName) => {
  const win = windows.get(panel);
  return win && !win.isDestroyed() ? win : null;
};

export const panelWindows = (): BrowserWindow[] =>
  [...windows.values()].filter((w) => !w.isDestroyed());

export const isPanelSender = (id: number) => panelWindows().some((w) => w.webContents.id === id);

export async function openPanelWindow(panel: DockPanelName, projectDir: string): Promise<void> {
  const { create, url } = panelHost();
  const existing = panelWindow(panel);
  if (existing) {
    raiseWindow(existing);
    return;
  }
  const saved = (await panelBounds())[panel];
  const size = DEFAULT_SIZE[panel];
  const rect = saved && isVisible(saved) ? saved : undefined;
  const win = create({ ...size, ...rect, title: `${TITLES[panel]} — Demoflow` });
  windows.set(panel, win);
  rememberBounds(win, (next) => savePanelBounds(panel, next));
  win.on("closed", () => {
    if (windows.get(panel) === win) windows.delete(panel);
    notify();
  });
  void win.loadURL(url(`/panel/${panel}?project=${encodeURIComponent(projectDir)}`));
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
