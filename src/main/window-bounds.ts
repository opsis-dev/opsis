import { eq } from "drizzle-orm";
import { screen, type BrowserWindow, type Rectangle } from "electron";
import { debounce } from "../shared/timing";
import { db } from "./db";
import { windowBounds } from "./db/schema";
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
