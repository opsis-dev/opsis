import { app, Menu, type BrowserWindow, type MenuItemConstructorOptions } from "electron";
import { IPC, type MenuCommand } from "../shared/ipc";

export function buildMenu(getWindow: () => BrowserWindow | null): void {
  const send = (command: MenuCommand) => () => getWindow()?.webContents.send(IPC.menu, command);
  const isMac = process.platform === "darwin";
  const template: MenuItemConstructorOptions[] = [
    ...(isMac ? [{ role: "appMenu" as const }] : []),
    {
      label: "File",
      submenu: [
        { label: "New project…", accelerator: "CmdOrCtrl+N", click: send("project:new") },
        { label: "Open project…", accelerator: "CmdOrCtrl+O", click: send("project:open") },
        { label: "Save", accelerator: "CmdOrCtrl+S", click: send("project:save") },
        { type: "separator" },
        { label: "All projects", accelerator: "CmdOrCtrl+Shift+P", click: send("view:projects") },
        { label: "Settings", accelerator: "CmdOrCtrl+,", click: send("view:settings") },
        { type: "separator" },
        isMac ? { role: "close" } : { role: "quit" },
      ],
    },
    {
      label: "Edit",
      submenu: [
        { label: "Undo", accelerator: "CmdOrCtrl+Z", click: send("edit:undo") },
        { label: "Redo", accelerator: isMac ? "Cmd+Shift+Z" : "Ctrl+Y", click: send("edit:redo") },
        { type: "separator" },
        { role: "cut" },
        { role: "copy" },
        { role: "paste" },
        { role: "selectAll" },
        { type: "separator" },
        { label: "Group selection", accelerator: "CmdOrCtrl+G", click: send("edit:group") },
        { label: "Tidy up layout", accelerator: "CmdOrCtrl+Shift+L", click: send("edit:layout") },
      ],
    },
    {
      label: "Run",
      submenu: [
        { label: "Run", accelerator: "CmdOrCtrl+Enter", click: send("run:start") },
        { label: "Cancel run", accelerator: "CmdOrCtrl+.", click: send("run:cancel") },
      ],
    },
    {
      label: "View",
      submenu: [
        { label: "Node list", accelerator: "CmdOrCtrl+B", click: send("view:toggle-left") },
        { label: "Preview and run", accelerator: "CmdOrCtrl+J", click: send("view:toggle-bottom") },
        { label: "Inspector", accelerator: "CmdOrCtrl+Alt+B", click: send("view:toggle-right") },
        { type: "separator" },
        { role: "reload" },
        { role: "toggleDevTools" },
        { type: "separator" },
        { role: "resetZoom" },
        { role: "zoomIn" },
        { role: "zoomOut" },
        { type: "separator" },
        { role: "togglefullscreen" },
      ],
    },
    { role: "windowMenu" },
    {
      role: "help",
      submenu: [{ label: `Demoflow ${app.getVersion()}`, enabled: false }],
    },
  ];
  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}
