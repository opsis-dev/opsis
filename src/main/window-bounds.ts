import { debounce } from "@opsis/core/timing";
import { eq } from "drizzle-orm";
import type { BrowserWindow, Rectangle } from "electron";
import { db } from "./db";
import { windowBounds } from "./db/schema";
import { raiseWindow } from "./raise-window";
import { SAVE_BOUNDS_AFTER_MS, visibleBounds } from "./window-placement";
import { makeWindow, rendererUrl, type WindowOptions } from "./window-registry";

function savedBounds(window: string): Rectangle | undefined {
  const row = db().select().from(windowBounds).where(eq(windowBounds.window, window)).get();
  return row && { x: row.x, y: row.y, width: row.width, height: row.height };
}

function saveBounds(window: string, rect: Rectangle): void {
  db()
    .insert(windowBounds)
    .values({ window, ...rect })
    .onConflictDoUpdate({ target: windowBounds.window, set: rect })
    .run();
}

function rememberBounds(win: BrowserWindow, save: (rect: Rectangle) => void): void {
  const remember = () => {
    if (!win.isDestroyed() && !win.isMinimized() && !win.isMaximized() && !win.isFullScreen())
      save(win.getBounds());
  };
  const rememberSoon = debounce(remember, SAVE_BOUNDS_AFTER_MS);
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
  options: WindowOptions;
  hash: string;
  onClosed: (win: BrowserWindow) => void;
};

const opening = new Map<string, Promise<BrowserWindow>>();

function createRememberedWindow(opts: RememberedWindow): BrowserWindow {
  const win = makeWindow({ ...opts.options, ...visibleBounds(savedBounds(opts.key)) });
  rememberBounds(win, (rect) => saveBounds(opts.key, rect));
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
  const created = Promise.resolve(createRememberedWindow(opts)).finally(() =>
    opening.delete(opts.key),
  );
  opening.set(opts.key, created);
  return created;
}
