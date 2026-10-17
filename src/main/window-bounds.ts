import { writeFile } from "node:fs/promises";
import path from "node:path";
import { app, screen, type BrowserWindow, type Rectangle } from "electron";
import { z } from "zod";
import { readJson } from "../engine/json";
import { debounce } from "../shared/timing";

const SAVE_AFTER_MS = 400;

const boundsFile = (name: string) => path.join(app.getPath("userData"), name);

export const rectangleSchema = z.object({
  x: z.number(),
  y: z.number(),
  width: z.number(),
  height: z.number(),
});

export function isVisible(rect: Rectangle): boolean {
  return screen.getAllDisplays().some((d) => {
    const a = d.workArea;
    return (
      rect.x < a.x + a.width - 40 &&
      rect.x + rect.width > a.x + 40 &&
      rect.y >= a.y - 20 &&
      rect.y < a.y + a.height - 40
    );
  });
}

export async function loadBounds<T>(name: string, schema: z.ZodType<T>): Promise<T | null> {
  return schema.safeParse(await readJson(boundsFile(name))).data ?? null;
}

export function saveBounds(name: string, value: unknown): void {
  void writeFile(boundsFile(name), JSON.stringify(value, null, 2)).catch(() => undefined);
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
