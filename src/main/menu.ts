import { app, Menu, type BrowserWindow, type MenuItemConstructorOptions } from "electron";
import { IPC } from "../shared/ipc";
import { appMenu } from "../shared/menu";

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
