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
        return { label: item.label, role: item.role, accelerator: item.accelerator };
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

export function runWindowRole(win: BrowserWindow, role: WindowRole): void {
  const contents = win.webContents;
  switch (role) {
    case "cut":
      contents.cut();
      return;
    case "copy":
      contents.copy();
      return;
    case "paste":
      contents.paste();
      return;
    case "selectAll":
      contents.selectAll();
      return;
    case "reload":
      contents.reload();
      return;
    case "toggleDevTools":
      contents.toggleDevTools();
      return;
    case "resetZoom":
      contents.setZoomLevel(0);
      return;
    case "zoomIn":
      contents.setZoomLevel(Math.min(contents.getZoomLevel() + 0.5, 5));
      return;
    case "zoomOut":
      contents.setZoomLevel(Math.max(contents.getZoomLevel() - 0.5, -5));
      return;
    case "togglefullscreen":
      win.setFullScreen(!win.isFullScreen());
      return;
    case "quit":
      app.quit();
      return;
  }
}
