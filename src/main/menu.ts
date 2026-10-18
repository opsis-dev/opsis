import { app, Menu, type MenuItemConstructorOptions } from "electron";
import { appMenu } from "../shared/menu";
import { sendMenuCommand } from "./window-registry";

export function buildMenu(): void {
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
        click: () => sendMenuCommand(item.command),
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
