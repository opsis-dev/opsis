import path from "node:path";
import { app, screen, type BrowserWindow, type Rectangle } from "electron";
import type { z } from "zod";
import { readJson, writeJson } from "../engine/json";
import { debounce } from "../shared/timing";
import { raiseWindow } from "./raise-window";
import { makeWindow, rendererUrl, type WindowOptions } from "./window-registry";

const SAVE_AFTER_MS = 400;
const MIN_VISIBLE_PX = 40;
const ABOVE_WORK_AREA_PX = 20;

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

const visibleBounds = (rect: Rectangle | null | undefined): Rectangle | undefined =>
  rect && isVisible(rect) ? rect : undefined;

export function boundsStore<T>(name: string, schema: z.ZodType<T>) {
  const file = () => path.join(app.getPath("userData"), name);
  return {
    load: (): Promise<T | null> => readJson(file(), schema),
    save: (value: T): void => void writeJson(file(), value).catch(() => undefined),
  };
}

function rememberBounds(win: BrowserWindow, save: (rect: Rectangle) => void): void {
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

type RememberedWindow = {
  key: string;
  existing: BrowserWindow | null;
  saved: () => Promise<Rectangle | null | undefined>;
  save: (rect: Rectangle) => void;
  options: WindowOptions;
  hash: string;
  onClosed: (win: BrowserWindow) => void;
};

const opening = new Map<string, Promise<BrowserWindow>>();

async function createRememberedWindow(opts: RememberedWindow): Promise<BrowserWindow> {
  const win = makeWindow({ ...opts.options, ...visibleBounds(await opts.saved()) });
  rememberBounds(win, opts.save);
  win.on("closed", () => opts.onClosed(win));
  void win.loadURL(rendererUrl(opts.hash));
  return win;
}

export function openRememberedWindow(opts: RememberedWindow): Promise<BrowserWindow | null> {
  const pending = opening.get(opts.key);
  if (pending)
    return pending.then((win) => {
      raiseWindow(win);
      return null;
    });
  if (opts.existing) {
    raiseWindow(opts.existing);
    return Promise.resolve(null);
  }
  const created = createRememberedWindow(opts).finally(() => opening.delete(opts.key));
  opening.set(opts.key, created);
  return created;
}
