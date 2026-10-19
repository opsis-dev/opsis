import { contextBridge, ipcRenderer, webUtils, type IpcRendererEvent } from "electron";
import type { ApiEditorReply, ApiEditorRequest } from "../shared/api/types";
import { errorMessage } from "../shared/errors";
import { INVOKE, IPC, type OpsisApi } from "../shared/ipc";
import { listenerSet } from "../shared/listener-set";
import { isLiveMessage, type LiveMessage } from "../shared/live";
import { mapValues } from "../shared/objects";

function subscribe<A extends unknown[]>(
  channel: string,
  listener: (...args: A) => void,
): () => void {
  const handler = (_event: IpcRendererEvent, ...args: A) => listener(...args);
  ipcRenderer.on(channel, handler);
  return () => ipcRenderer.removeListener(channel, handler);
}

const live = listenerSet<[LiveMessage]>();

ipcRenderer.on(IPC.livePort, (event) => {
  const [port] = event.ports;
  port.addEventListener("message", (message) => {
    if (isLiveMessage(message.data)) live.emit(message.data);
  });
  port.start();
});

const invokers = mapValues(
  INVOKE,
  (channel) =>
    (...args: unknown[]) =>
      ipcRenderer.invoke(channel, ...args),
);

const answerApiRequest = (
  listener: (request: ApiEditorRequest) => Promise<ApiEditorReply>,
  id: string,
  request: ApiEditorRequest,
) =>
  void listener(request)
    .catch((e: unknown): ApiEditorReply => ({ ok: false, status: 500, error: errorMessage(e) }))
    .then((reply) => ipcRenderer.send(IPC.apiReply, id, reply));

const api: OpsisApi = {
  ...invokers,
  platform: process.platform,
  pathForFile: (file) => webUtils.getPathForFile(file),
  onProjectFileChanged: (listener) => subscribe(IPC.projectFileChanged, listener),
  onRunEvent: (listener) => subscribe(IPC.runEvent, listener),
  onEngineInstall: (listener) => subscribe(IPC.engineEvent, listener),
  onSettingsChanged: (listener) => subscribe(IPC.settingsChanged, listener),
  onApiStatus: (listener) => subscribe(IPC.apiStatusChanged, listener),
  onApiRequest: (listener) => {
    const off = subscribe(IPC.apiRequest, (id: string, request: ApiEditorRequest) =>
      answerApiRequest(listener, id, request),
    );
    ipcRenderer.send(IPC.apiListening, true);
    return () => {
      off();
      ipcRenderer.send(IPC.apiListening, false);
    };
  },
  onLive: (listener) => live.add(listener),
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
  onSettingsPage: (listener) => subscribe(IPC.settingsPage, listener),
  setTitle: (title) => ipcRenderer.send(IPC.setTitle, title),
};

contextBridge.exposeInMainWorld("opsis", api);
