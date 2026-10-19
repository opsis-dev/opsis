import type { BrowserWindow } from "electron";
import { z } from "zod";
import type { ViewPrefs } from "../shared/ipc";
import { debounce } from "../shared/timing";
import { writeViewState } from "./view-state";
import { SAVE_BOUNDS_AFTER_MS, visibleBounds } from "./window-placement";

const MAIN_WINDOW_KEY = "mainWindow";
const EDITOR_ROUTE = "/editor";

const windowStateSchema = z.object({
  x: z.number(),
  y: z.number(),
  width: z.number(),
  height: z.number(),
  maximized: z.boolean(),
});

export function shownProjectDir(url: string): string | null {
  const route = url.split("#")[1] ?? "";
  const [path = "", query = ""] = route.split("?");
  if (path !== EDITOR_ROUTE) return null;
  return new URLSearchParams(query).get("dir") || null;
}

export function applyProjectWindow(win: BrowserWindow, view: ViewPrefs): void {
  if (win.isFullScreen()) return;
  const saved = windowStateSchema.safeParse(view[MAIN_WINDOW_KEY]);
  if (!saved.success) {
    win.maximize();
    return;
  }
  const { maximized, ...rect } = saved.data;
  if (win.isMaximized()) win.unmaximize();
  const bounds = visibleBounds(rect);
  if (bounds) win.setBounds(bounds);
  if (maximized) win.maximize();
}

export function rememberProjectWindow(win: BrowserWindow): void {
  const save = () => {
    if (win.isDestroyed() || win.isMinimized() || win.isFullScreen()) return;
    const dir = shownProjectDir(win.webContents.getURL());
    if (dir)
      writeViewState(dir, MAIN_WINDOW_KEY, {
        ...win.getNormalBounds(),
        maximized: win.isMaximized(),
      });
  };
  const saveSoon = debounce(save, SAVE_BOUNDS_AFTER_MS);
  win.on("resized", saveSoon);
  win.on("moved", saveSoon);
  win.on("maximize", saveSoon);
  win.on("unmaximize", saveSoon);
  win.on("close", () => {
    saveSoon.cancel();
    save();
  });
}
