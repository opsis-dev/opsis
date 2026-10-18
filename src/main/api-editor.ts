import { randomUUID } from "node:crypto";
import type { IncomingMessage, ServerResponse } from "node:http";
import { ipcMain, type WebContents } from "electron";
import {
  apiFailure,
  apiOk,
  apiProjectStateSchema,
  type ApiEditorReply,
  type ApiEditorRequest,
  type ApiProjectState,
  type ApiReply,
} from "../shared/api/types";
import { NO_PROJECT } from "../shared/errors";
import { IPC } from "../shared/ipc";
import { listenerSet } from "../shared/listener-set";
import { pendingReplies } from "../shared/pending-replies";
import type { RunEvent } from "../shared/schema/events";
import { getMainWindow, isEditorSender } from "./window-registry";

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
const editorListens = listenerSet();
let editorListening = false;

function setEditorListening(listening: boolean): void {
  editorListening = listening;
  if (listening) editorListens.emit();
}

function untilEditorListens(ms: number): Promise<boolean> {
  if (editorListening) return Promise.resolve(true);
  return new Promise((resolve) => {
    const done = (listens: boolean) => {
      clearTimeout(timer);
      stop();
      resolve(listens);
    };
    const timer = setTimeout(() => done(false), ms);
    const stop = editorListens.add(() => done(true));
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

export async function askEditor(request: ApiEditorRequest): Promise<ApiEditorReply> {
  const win = getMainWindow();
  if (!win || win.isDestroyed()) return NO_EDITOR;
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
