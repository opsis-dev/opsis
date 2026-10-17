import { app, BrowserWindow } from "electron";
import started from "electron-squirrel-startup";
import { IPC } from "../shared/ipc";
import { stopAnalysis } from "./analysis";
import { followApiSetting, stopApi } from "./api";
import { registerApiEditorIpc } from "./api-editor";
import {
  broadcast,
  createMainWindow,
  getMainWindow,
  isEditorSender,
  makeWindow,
  rendererDir,
  rendererUrl,
} from "./app-windows";
import { completeLicences } from "./engine-setup";
import { interceptQuit, registerExitIpc } from "./exit";
import { fontsDir } from "./fonts";
import { currentProjectDir, registerIpc } from "./ipc-handlers";
import { buildMenu } from "./menu";
import { initPanelWindows } from "./panel-windows";
import { registerProtocolHandlers, registerSchemes } from "./protocol";
import { onSettingsChange } from "./settings";
import { initSettingsWindow } from "./settings-window";
import { stopStills } from "./stills";
import { clearStaleWorkerTemps } from "./worker-temp";
import { cancelRun, workerTempRoot } from "./workers";

if (started) app.quit();

registerSchemes();

app.whenReady().then(() => {
  clearStaleWorkerTemps(workerTempRoot());
  registerProtocolHandlers({
    getProjectDir: currentProjectDir,
    getFontsDir: fontsDir,
    rendererDir,
  });
  registerIpc();
  registerExitIpc(isEditorSender);
  initPanelWindows({
    main: getMainWindow,
    create: (o) => makeWindow({ ...o, minWidth: 420, minHeight: 260 }),
    url: rendererUrl,
  });
  initSettingsWindow({ create: makeWindow, url: rendererUrl });
  registerApiEditorIpc();
  onSettingsChange((settings) => broadcast(IPC.settingsChanged, settings));
  followApiSetting();
  void completeLicences();
  buildMenu();
  createMainWindow();
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createMainWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

app.on("before-quit", (event) => interceptQuit(event, getMainWindow()));

app.on("will-quit", () => {
  cancelRun();
  stopAnalysis();
  stopStills();
  stopApi();
});
