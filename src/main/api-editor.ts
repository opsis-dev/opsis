import { randomUUID } from "node:crypto";
import type { IncomingMessage, ServerResponse } from "node:http";
import { ipcMain } from "electron";
import type { ApiEditorReply, ApiEditorRequest } from "../shared/api/types";
import { IPC } from "../shared/ipc";
import type { RunEvent } from "../shared/schema/events";
import { getMainWindow, isEditorSender } from "./app-windows";

const EDITOR_TIMEOUT_MS = 20_000;

const pending = new Map<string, (reply: ApiEditorReply) => void>();
const streams = new Set<ServerResponse>();

export function registerApiEditorIpc(): void {
  ipcMain.on(IPC.apiReply, (event, id: string, answer: ApiEditorReply) => {
    if (!isEditorSender(event.sender.id)) return;
    pending.get(id)?.(answer);
    pending.delete(id);
  });
}

export function askEditor<R extends ApiEditorRequest>(
  request: R,
): Promise<ApiEditorReply<R["op"]>> {
  const win = getMainWindow();
  if (!win || win.isDestroyed())
    return Promise.resolve({ ok: false, status: 409, error: "No project is open." });
  const id = randomUUID();
  return new Promise((resolve) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      resolve({ ok: false, status: 504, error: "The editor didn't answer in time." });
    }, EDITOR_TIMEOUT_MS);
    pending.set(id, (answer) => {
      clearTimeout(timer);
      resolve(answer as ApiEditorReply<R["op"]>);
    });
    win.webContents.send(IPC.apiRequest, id, request);
  });
}

export function followRunEvents(req: IncomingMessage, res: ServerResponse): void {
  res.writeHead(200, {
    "content-type": "text/event-stream",
    "cache-control": "no-cache",
    connection: "keep-alive",
  });
  res.write(": following run events\n\n");
  streams.add(res);
  req.on("close", () => streams.delete(res));
}

export function apiRunEvent(event: RunEvent): void {
  for (const stream of streams) stream.write(`data: ${JSON.stringify(event)}\n\n`);
}

export function endRunEventStreams(): void {
  for (const stream of streams) stream.end();
  streams.clear();
}
