import { app, BrowserWindow } from "electron";
import started from "electron-squirrel-startup";
import { IPC } from "../shared/ipc";
import { stopAnalysis } from "./analysis";
import { initApi, setApiEnabled, stopApi } from "./api";
import {
  broadcast,
  createMainWindow,
  getMainWindow,
  isEditorSender,
  makeWindow,
  rendererDir,
  rendererUrl,
} from "./app-windows";
import { interceptQuit, registerExitIpc } from "./exit";
import { fontsDir } from "./fonts";
import { currentProjectDir, registerIpc } from "./ipc-handlers";
import { buildMenu } from "./menu";
import { initPanelWindows } from "./panel-windows";
import { registerProtocolHandlers, registerSchemes } from "./protocol";
import { onSettingsChange, patchSettings, readSettings } from "./settings";
import { initSettingsWindow } from "./settings-window";
import { initStills, stopStills } from "./stills";
import { clearStaleWorkerTemps } from "./worker-temp";
import { cancelRun, completeLicences, isRunning, workerTempRoot } from "./workers";

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
  initStills(getMainWindow);
  initPanelWindows({
    main: getMainWindow,
    create: (o) => makeWindow({ ...o, minWidth: 420, minHeight: 260 }),
    url: rendererUrl,
  });
  initSettingsWindow({ create: makeWindow, url: rendererUrl });
  initApi(
    {
      editor: getMainWindow,
      onStatus: (status) => broadcast(IPC.apiStatusChanged, status),
    },
    isEditorSender,
  );
  onSettingsChange((settings, previous) => {
    broadcast(IPC.settingsChanged, settings);
    if (settings.api.enabled !== previous.api.enabled) void setApiEnabled(settings.api.enabled);
    if (settings.projects.defaultDir !== previous.projects.defaultDir && settings.projects.lastDir)
      void patchSettings({ projects: { lastDir: "" } });
  });
  void readSettings().then((settings) => setApiEnabled(settings.api.enabled));
  void completeLicences();
  buildMenu(getMainWindow);
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
  if (isRunning()) cancelRun();
  stopAnalysis();
  stopStills();
  stopApi();
});
