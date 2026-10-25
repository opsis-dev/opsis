import { randomUUID } from "node:crypto";
import type { IncomingMessage, ServerResponse } from "node:http";
import {
  API_EDITOR_TIMEOUT_MS,
  apiFailure,
  apiOk,
  apiProjectStateSchema,
  type ApiEditorReply,
  type ApiEditorRequest,
  type ApiProjectState,
  type ApiReply,
} from "@opsis/core/api/types";
import { NO_PROJECT } from "@opsis/core/errors";
import { IPC } from "@opsis/core/ipc";
import { listenerSet } from "@opsis/core/listener-set";
import { pendingReplies } from "@opsis/core/pending-replies";
import type { RunEvent } from "@opsis/core/schema/events";
import { whenReady } from "@opsis/core/timing";
import { ipcMain, type WebContents } from "electron";
import { getMainWindow, isEditorSender, liveWindow } from "./window-registry";

const EDITOR_TIMEOUT = {
  ms: API_EDITOR_TIMEOUT_MS,
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
const editorListens = listenerSet();
let editorListening = false;

function setEditorListening(listening: boolean): void {
  editorListening = listening;
  if (listening) editorListens.emit();
}

const untilEditorListens = (ms: number): Promise<true | null> =>
  whenReady(() => editorListening || null, editorListens.add, ms);

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

export async function askEditor(request: ApiEditorRequest): Promise<ApiEditorReply> {
  const win = liveWindow(getMainWindow());
  if (!win) return NO_EDITOR;
  if (!(await untilEditorListens(EDITOR_TIMEOUT.ms))) return EDITOR_TIMEOUT.result;
  const id = randomUUID();
  const reply = replies.wait(id, EDITOR_TIMEOUT);
  win.webContents.send(IPC.apiRequest, id, request);
  return reply;
}

export async function readEditorProject(): Promise<ApiReply<ApiProjectState>> {
  const reply = await askEditor({ op: "read" });
  if (!reply.ok) return reply;
  const project = apiProjectStateSchema.safeParse(reply.result);
  return project.success
    ? apiOk(project.data)
    : apiFailure(502, "The editor's answer couldn't be read.");
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
