import { contextBridge, ipcRenderer, webUtils, type IpcRendererEvent } from "electron";
import {
  IPC,
  type DemoflowApi,
  type EngineInstallEvent,
  type MenuCommand,
  type StillReady,
  type WindowState,
} from "../shared/ipc";
import type { LiveMessage } from "../shared/live";
import type { RunEvent } from "../shared/schema/events";

function subscribe<T>(channel: string, listener: (payload: T) => void): () => void {
  const handler = (_event: IpcRendererEvent, payload: T) => listener(payload);
  ipcRenderer.on(channel, handler);
  return () => ipcRenderer.removeListener(channel, handler);
}

const liveListeners = new Set<(message: LiveMessage) => void>();
const menuListeners = new Set<(command: MenuCommand) => void>();

ipcRenderer.on(IPC.menu, (_event, command: MenuCommand) => {
  for (const listener of menuListeners) listener(command);
});

ipcRenderer.on(IPC.livePort, (event) => {
  const [port] = event.ports;
  port.addEventListener("message", (message) => {
    for (const listener of liveListeners) listener(message.data as LiveMessage);
  });
  port.start();
});

const api: DemoflowApi = {
  platform: process.platform,
  listRecentProjects: () => ipcRenderer.invoke(IPC.listRecent),
  createProject: (opts) => ipcRenderer.invoke(IPC.createProject, opts),
  openProject: (dir) => ipcRenderer.invoke(IPC.openProject, dir),
  saveWorkflow: (dir, workflow) => ipcRenderer.invoke(IPC.saveWorkflow, dir, workflow),
  readLastRun: (dir) => ipcRenderer.invoke(IPC.readLastRun, dir),
  importAssets: (dir, paths) => ipcRenderer.invoke(IPC.importAssets, dir, paths),
  pathForFile: (file) => webUtils.getPathForFile(file),
  revealInFolder: (dir, rel) => ipcRenderer.invoke(IPC.reveal, dir, rel),
  runWorkflow: (req) => ipcRenderer.invoke(IPC.run, req),
  cancelRun: () => ipcRenderer.invoke(IPC.cancel),
  runStatus: () => ipcRenderer.invoke(IPC.runStatus),
  onRunEvent: (listener) => subscribe<RunEvent>(IPC.runEvent, listener),
  pickElement: (req) => ipcRenderer.invoke(IPC.pick, req),
  getEnvironment: () => ipcRenderer.invoke(IPC.env),
  installEngine: () => ipcRenderer.invoke(IPC.installEngine),
  onEngineInstall: (listener) => subscribe<EngineInstallEvent>(IPC.engineEvent, listener),
  getSettings: () => ipcRenderer.invoke(IPC.getSettings),
  setSettings: (settings) => ipcRenderer.invoke(IPC.setSettings, settings),
  onLive: (listener) => {
    liveListeners.add(listener);
    return () => liveListeners.delete(listener);
  },
  readTrace: (dir, traceId) => ipcRenderer.invoke(IPC.readTrace, dir, traceId),
  listSecrets: (dir) => ipcRenderer.invoke(IPC.listSecrets, dir),
  listFonts: () => ipcRenderer.invoke(IPC.listFonts),
  inspectFontFiles: (paths) => ipcRenderer.invoke(IPC.inspectFontFiles, paths),
  inspectInstalledFonts: (faces) => ipcRenderer.invoke(IPC.inspectInstalledFonts, faces),
  addInspectedFonts: (token, families) =>
    ipcRenderer.invoke(IPC.addInspectedFonts, token, families),
  removeFont: (id) => ipcRenderer.invoke(IPC.removeFont, id),
  fontCatalog: () => ipcRenderer.invoke(IPC.fontCatalog),
  downloadFont: (id) => ipcRenderer.invoke(IPC.downloadFont, id),
  useFontInProject: (dir, fontId) => ipcRenderer.invoke(IPC.useFontInProject, dir, fontId),
  clearCache: (dir) => ipcRenderer.invoke(IPC.clearCache, dir),
  onMenuCommand: (listener) => {
    menuListeners.add(listener);
    return () => menuListeners.delete(listener);
  },
  dispatchMenuCommand: (command) => {
    for (const listener of menuListeners) listener(command);
  },
  windowRole: (role) => ipcRenderer.invoke(IPC.windowRole, role),
  getWindowState: () => ipcRenderer.invoke(IPC.getWindowState),
  requestStill: (request) => ipcRenderer.invoke(IPC.requestStill, request),
  listStills: (dir) => ipcRenderer.invoke(IPC.listStills, dir),
  onStillReady: (listener) => subscribe<StillReady>(IPC.stillReady, listener),
  onWindowState: (listener) => subscribe<WindowState>(IPC.windowState, listener),
  setTitle: (title) => ipcRenderer.send(IPC.setTitle, title),
};

contextBridge.exposeInMainWorld("demoflow", api);
