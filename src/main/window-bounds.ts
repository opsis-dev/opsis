import path from "node:path";
import { app, screen, type BrowserWindow, type Rectangle } from "electron";
import { z } from "zod";
import { readJson, writeJson } from "../engine/json";
import { debounce } from "../shared/timing";

const SAVE_AFTER_MS = 400;
const MIN_VISIBLE_PX = 40;
const ABOVE_WORK_AREA_PX = 20;

export const rectangleSchema = z.object({
  x: z.number(),
  y: z.number(),
  width: z.number(),
  height: z.number(),
});

function isVisible(rect: Rectangle): boolean {
  return screen.getAllDisplays().some(({ workArea: area }) => {
    const overlapsAcross =
      rect.x < area.x + area.width - MIN_VISIBLE_PX &&
      rect.x + rect.width > area.x + MIN_VISIBLE_PX;
    const titleBarReachable =
      rect.y >= area.y - ABOVE_WORK_AREA_PX && rect.y < area.y + area.height - MIN_VISIBLE_PX;
    return overlapsAcross && titleBarReachable;
  });
}

export const visibleBounds = (rect: Rectangle | null | undefined): Rectangle | undefined =>
  rect && isVisible(rect) ? rect : undefined;

export function boundsStore<T>(name: string, schema: z.ZodType<T>) {
  const file = () => path.join(app.getPath("userData"), name);
  return {
    load: (): Promise<T | null> => readJson(file(), schema),
    save: (value: T): void => void writeJson(file(), value).catch(() => undefined),
  };
}

export function rememberBounds(win: BrowserWindow, save: (rect: Rectangle) => void): void {
  const remember = () => {
    if (!win.isDestroyed() && !win.isMinimized() && !win.isMaximized() && !win.isFullScreen())
      save(win.getBounds());
  };
  const rememberSoon = debounce(remember, SAVE_AFTER_MS);
  win.on("resized", rememberSoon);
  win.on("moved", rememberSoon);
  win.on("close", () => {
    rememberSoon.cancel();
    remember();
  });
}

export function requireHost<T>(host: T | null, what: string): T {
  if (!host) throw new Error(`${what} opened before the app finished starting.`);
  return host;
}

export const liveWindow = (win: BrowserWindow | null | undefined): BrowserWindow | null =>
  win && !win.isDestroyed() ? win : null;
