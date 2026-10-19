import { app, BrowserWindow } from "electron";
import started from "electron-squirrel-startup";
import { IPC } from "../shared/ipc";
import { stopAnalysis } from "./analysis";
import { followApiSetting, stopApi } from "./api";
import { registerApiEditorIpc } from "./api-editor";
import { broadcast, createMainWindow } from "./app-windows";
import { closeDatabase, openDatabase } from "./db";
import { interceptQuit, registerExitIpc } from "./exit";
import { fontsDir } from "./fonts";
import { registerIpc } from "./ipc-handlers";
import { completeLicences } from "./licences";
import { buildMenu } from "./menu";
import { currentProjectDir } from "./projects";
import { registerProtocolHandlers, registerSchemes } from "./protocol";
import { cancelRun } from "./run-host";
import { onSettingsChange } from "./settings";
import { stopStills } from "./stills";
import { getMainWindow, rendererDir } from "./window-registry";
import { clearStaleWorkerTemps } from "./worker-temp";
import { workerTempRoot } from "./workers";

if (started) app.quit();

registerSchemes();

app.whenReady().then(() => {
  openDatabase();
  clearStaleWorkerTemps(workerTempRoot());
  registerProtocolHandlers({
    getProjectDir: currentProjectDir,
    getFontsDir: fontsDir,
    rendererDir,
  });
  registerIpc();
  registerExitIpc();
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
  void stopApi();
  closeDatabase();
});
