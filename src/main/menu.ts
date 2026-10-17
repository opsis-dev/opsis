import { app, Menu, type BrowserWindow, type MenuItemConstructorOptions } from "electron";
import { IPC } from "../shared/ipc";
import { appMenu, type WindowRole } from "../shared/menu";

export function buildMenu(getWindow: () => BrowserWindow | null): void {
  if (process.platform !== "darwin") {
    Menu.setApplicationMenu(null);
    return;
  }
  const sections: MenuItemConstructorOptions[] = appMenu(process.platform).map((section) => ({
    label: section.label,
    submenu: section.items.map((item): MenuItemConstructorOptions => {
      if (item.kind === "separator") return { type: "separator" };
      if (item.kind === "role")
        return {
          label: item.label,
          role: item.role === "toggleMaximize" ? "zoom" : item.role,
          accelerator: item.accelerator,
        };
      return {
        label: item.label,
        accelerator: item.accelerator,
        click: () => getWindow()?.webContents.send(IPC.menu, item.command),
      };
    }),
  }));
  Menu.setApplicationMenu(
    Menu.buildFromTemplate([
      { role: "appMenu" },
      ...sections,
      { role: "windowMenu" },
      { role: "help", submenu: [{ label: `Demoflow ${app.getVersion()}`, enabled: false }] },
    ]),
  );
}

const ZOOM_STEP = 0.5;
const ZOOM_LIMIT = 5;

const ROLE_ACTIONS: Record<WindowRole, (win: BrowserWindow) => void> = {
  cut: (win) => win.webContents.cut(),
  copy: (win) => win.webContents.copy(),
  paste: (win) => win.webContents.paste(),
  selectAll: (win) => win.webContents.selectAll(),
  reload: (win) => win.webContents.reload(),
  toggleDevTools: (win) => win.webContents.toggleDevTools(),
  resetZoom: (win) => win.webContents.setZoomLevel(0),
  zoomIn: (win) =>
    win.webContents.setZoomLevel(Math.min(win.webContents.getZoomLevel() + ZOOM_STEP, ZOOM_LIMIT)),
  zoomOut: (win) =>
    win.webContents.setZoomLevel(Math.max(win.webContents.getZoomLevel() - ZOOM_STEP, -ZOOM_LIMIT)),
  togglefullscreen: (win) => win.setFullScreen(!win.isFullScreen()),
  minimize: (win) => win.minimize(),
  toggleMaximize: (win) => (win.isMaximized() ? win.unmaximize() : win.maximize()),
  close: (win) => win.close(),
  quit: () => app.quit(),
};

export function runWindowRole(win: BrowserWindow, role: WindowRole): void {
  ROLE_ACTIONS[role](win);
}
