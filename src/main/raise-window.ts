import type { BrowserWindow } from "electron";

export function raiseWindow(win: BrowserWindow): void {
  if (win.isMinimized()) win.restore();
  win.show();
  win.focus();
}
