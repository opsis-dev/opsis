import { randomUUID } from "node:crypto";
import type { IncomingMessage, ServerResponse } from "node:http";
import { ipcMain, type WebContents } from "electron";
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

const RELOADED = {
  ok: false,
  status: 503,
  error: "The editor reloaded before it answered.",
} as const;

const replies = pendingReplies<ApiEditorReply>();
const streams = new Set<ServerResponse>();
const watched = new WeakSet<WebContents>();
const wakeOnListening = new Set<() => void>();
let editorListening = false;

function setEditorListening(listening: boolean): void {
  editorListening = listening;
  if (listening) for (const wake of wakeOnListening) wake();
}

function untilEditorListens(ms: number): Promise<boolean> {
  if (editorListening) return Promise.resolve(true);
  return new Promise((resolve) => {
    const done = (listening: boolean) => {
      clearTimeout(timer);
      wakeOnListening.delete(wake);
      resolve(listening);
    };
    const wake = () => done(true);
    const timer = setTimeout(() => done(false), ms);
    wakeOnListening.add(wake);
  });
}

function watchEditor(contents: WebContents): void {
  if (watched.has(contents)) return;
  watched.add(contents);
  contents.on("did-start-navigation", ({ isMainFrame, isSameDocument }) => {
    if (!isMainFrame || isSameDocument) return;
    setEditorListening(false);
    replies.settleAll(RELOADED);
  });
  contents.once("destroyed", () => {
    setEditorListening(false);
    replies.settleAll(NO_EDITOR);
  });
}

export function registerApiEditorIpc(): void {
  ipcMain.on(IPC.apiReply, (event, id: string, answer: ApiEditorReply) => {
    if (isEditorSender(event.sender.id)) replies.settle(id, answer);
  });
  ipcMain.on(IPC.apiListening, (event, listening: boolean) => {
    if (!isEditorSender(event.sender.id)) return;
    watchEditor(event.sender);
    setEditorListening(listening);
  });
}

export async function askEditor<R extends ApiEditorRequest>(
  request: R,
): Promise<ApiEditorReply<R["op"]>> {
  const win = getMainWindow();
  if (!win || win.isDestroyed()) return NO_EDITOR;
  if (!(await untilEditorListens(EDITOR_TIMEOUT.ms))) return EDITOR_TIMEOUT.result;
  const id = randomUUID();
  const reply = replies.wait(id, EDITOR_TIMEOUT);
  win.webContents.send(IPC.apiRequest, id, request);
  return (await reply) as ApiEditorReply<R["op"]>;
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
