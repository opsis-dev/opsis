import { contextBridge, ipcRenderer, webUtils, type IpcRendererEvent } from "electron";
import { INVOKE, IPC, type DemoflowApi, type InvokeName } from "../shared/ipc";
import type { LiveMessage } from "../shared/live";
import type { ApiEditorReply, ApiEditorRequest } from "../shared/api/types";
import { errorMessage } from "../shared/errors";

function subscribe<A extends unknown[]>(
  channel: string,
  listener: (...args: A) => void,
): () => void {
  const handler = (_event: IpcRendererEvent, ...args: A) => listener(...args);
  ipcRenderer.on(channel, handler);
  return () => ipcRenderer.removeListener(channel, handler);
}

const liveListeners = new Set<(message: LiveMessage) => void>();

ipcRenderer.on(IPC.livePort, (event) => {
  const [port] = event.ports;
  port.addEventListener("message", (message) => {
    for (const listener of liveListeners) listener(message.data as LiveMessage);
  });
  port.start();
});

const invokers = Object.fromEntries(
  Object.entries(INVOKE).map(([name, channel]) => [
    name,
    (...args: unknown[]) => ipcRenderer.invoke(channel, ...args),
  ]),
) as Pick<DemoflowApi, InvokeName>;

const answerApiRequest = (
  listener: (request: ApiEditorRequest) => Promise<ApiEditorReply>,
  id: string,
  request: ApiEditorRequest,
) =>
  void listener(request)
    .catch((e: unknown): ApiEditorReply => ({ ok: false, status: 500, error: errorMessage(e) }))
    .then((reply) => ipcRenderer.send(IPC.apiReply, id, reply));

const api: DemoflowApi = {
  ...invokers,
  platform: process.platform,
  pathForFile: (file) => webUtils.getPathForFile(file),
  onProjectFileChanged: (listener) => subscribe(IPC.projectFileChanged, listener),
  onRunEvent: (listener) => subscribe(IPC.runEvent, listener),
  onEngineInstall: (listener) => subscribe(IPC.engineEvent, listener),
  onSettingsChanged: (listener) => subscribe(IPC.settingsChanged, listener),
  onApiStatus: (listener) => subscribe(IPC.apiStatusChanged, listener),
  onApiRequest: (listener) =>
    subscribe(IPC.apiRequest, (id: string, request: ApiEditorRequest) =>
      answerApiRequest(listener, id, request),
    ),
  onLive: (listener) => {
    liveListeners.add(listener);
    return () => liveListeners.delete(listener);
  },
  onMenuCommand: (listener) => subscribe(IPC.menu, listener),
  sendMenuToEditor: (command) => ipcRenderer.send(IPC.menuToEditor, command),
  onPanelWindows: (listener) => subscribe(IPC.panelWindows, listener),
  reportUnsaved: (unsaved) => ipcRenderer.send(IPC.reportUnsaved, unsaved),
  onConfirmClose: (listener) => subscribe(IPC.confirmClose, listener),
  decideClose: (decision) => ipcRenderer.send(IPC.closeDecision, decision),
  closeShown: () => ipcRenderer.send(IPC.closeShown),
  onFlushSave: (listener) =>
    subscribe(IPC.flushSave, () => {
      void listener()
        .catch((e: unknown) => console.error("The project couldn't be saved before closing.", e))
        .finally(() => ipcRenderer.send(IPC.saveFlushed));
    }),
  onStillReady: (listener) => subscribe(IPC.stillReady, listener),
  onWindowState: (listener) => subscribe(IPC.windowState, listener),
  setTitle: (title) => ipcRenderer.send(IPC.setTitle, title),
};

contextBridge.exposeInMainWorld("demoflow", api);
