import { app, type BrowserWindow } from "electron";
import { clamp } from "../shared/math";
import type { WindowRole } from "../shared/menu";

const ZOOM_STEP = 0.5;
const ZOOM_LIMIT = 5;

const zoomBy = (win: BrowserWindow, step: number) =>
  win.webContents.setZoomLevel(
    clamp(win.webContents.getZoomLevel() + step, -ZOOM_LIMIT, ZOOM_LIMIT),
  );

const ROLE_ACTIONS: Record<WindowRole, (win: BrowserWindow) => void> = {
  cut: (win) => win.webContents.cut(),
  copy: (win) => win.webContents.copy(),
  paste: (win) => win.webContents.paste(),
  selectAll: (win) => win.webContents.selectAll(),
  reload: (win) => win.webContents.reload(),
  toggleDevTools: (win) => win.webContents.toggleDevTools(),
  resetZoom: (win) => win.webContents.setZoomLevel(0),
  zoomIn: (win) => zoomBy(win, ZOOM_STEP),
  zoomOut: (win) => zoomBy(win, -ZOOM_STEP),
  togglefullscreen: (win) => win.setFullScreen(!win.isFullScreen()),
  minimize: (win) => win.minimize(),
  toggleMaximize: (win) => (win.isMaximized() ? win.unmaximize() : win.maximize()),
  close: (win) => win.close(),
  quit: () => app.quit(),
};

export function runWindowRole(win: BrowserWindow, role: WindowRole): void {
  ROLE_ACTIONS[role](win);
}
