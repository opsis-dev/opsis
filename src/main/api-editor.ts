import { randomUUID } from "node:crypto";
import type { IncomingMessage, ServerResponse } from "node:http";
import { ipcMain } from "electron";
import type { ApiEditorReply, ApiEditorRequest } from "../shared/api/types";
import { NO_PROJECT } from "../shared/errors";
import { IPC } from "../shared/ipc";
import { pendingReplies } from "../shared/pending-replies";
import type { RunEvent } from "../shared/schema/events";
import { getMainWindow, isEditorSender } from "./app-windows";

const EDITOR_TIMEOUT = {
  ms: 20_000,
  result: { ok: false, status: 504, error: "The editor didn't answer in time." },
} as const;

const NO_EDITOR = { ok: false, status: 409, error: NO_PROJECT } as const;

const replies = pendingReplies<ApiEditorReply>();
const streams = new Set<ServerResponse>();

export function registerApiEditorIpc(): void {
  ipcMain.on(IPC.apiReply, (event, id: string, answer: ApiEditorReply) => {
    if (isEditorSender(event.sender.id)) replies.settle(id, answer);
  });
}

export async function askEditor<R extends ApiEditorRequest>(
  request: R,
): Promise<ApiEditorReply<R["op"]>> {
  const win = getMainWindow();
  if (!win || win.isDestroyed()) return NO_EDITOR;
  const id = randomUUID();
  const closed = () => replies.settle(id, NO_EDITOR);
  win.once("closed", closed);
  const reply = replies.wait(id, EDITOR_TIMEOUT);
  win.webContents.send(IPC.apiRequest, id, request);
  const answer = await reply;
  win.removeListener("closed", closed);
  return answer as ApiEditorReply<R["op"]>;
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
