import { writeFile } from "node:fs/promises";
import path from "node:path";
import { app, screen, type BrowserWindow, type Rectangle } from "electron";
import { readJson } from "../engine/json";

const boundsFile = (name: string) => path.join(app.getPath("userData"), name);

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

export const loadBounds = <T>(name: string): Promise<T | null> => readJson<T>(boundsFile(name));

export function saveBounds(name: string, value: unknown): void {
  void writeFile(boundsFile(name), JSON.stringify(value, null, 2)).catch(() => undefined);
}

export function rememberBounds(win: BrowserWindow, save: (rect: Rectangle) => void): void {
  const remember = () => {
    if (!win.isDestroyed() && !win.isMinimized() && !win.isMaximized() && !win.isFullScreen())
      save(win.getBounds());
  };
  win.on("resized", remember);
  win.on("moved", remember);
  win.on("close", remember);
}
