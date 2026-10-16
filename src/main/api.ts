import { randomUUID } from "node:crypto";
import { existsSync } from "node:fs";
import { chmod, rm } from "node:fs/promises";
import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import { connect } from "node:net";
import { userInfo } from "node:os";
import path from "node:path";
import { app, ipcMain, type BrowserWindow } from "electron";
import { catalogEntry, nodeCatalog } from "../shared/api/catalog";
import { checkWorkflow } from "../shared/api/check";
import { openApiDocument } from "../shared/api/openapi";
import type { ApiEditorReply, ApiEditorRequest } from "../shared/api/types";
import { IPC, type ApiStatus } from "../shared/ipc";
import type { RunEvent } from "../shared/schema/events";
import { WORKFLOW_FILE } from "../shared/schema/workflow";

const MAX_BODY = 20 * 1024 * 1024;
const EDITOR_TIMEOUT_MS = 20_000;

export const apiAddress = () =>
  process.platform === "win32"
    ? `\\\\.\\pipe\\demoflow-api-${userInfo().username}`
    : path.join(app.getPath("userData"), "api.sock");

type Host = {
  editor: () => BrowserWindow | null;
  onStatus: (status: ApiStatus) => void;
};

let host: Host | null = null;
let server: Server | null = null;
let status: ApiStatus = { enabled: false, listening: false, address: "" };
const pending = new Map<string, (reply: ApiEditorReply) => void>();
const streams = new Set<ServerResponse>();

export function initApi(next: Host, isTrusted: (id: number) => boolean): void {
  host = next;
  status = { ...status, address: apiAddress() };
  ipcMain.on(IPC.apiReply, (event, id: string, answer: ApiEditorReply) => {
    if (!isTrusted(event.sender.id)) return;
    pending.get(id)?.(answer);
    pending.delete(id);
  });
}

export const getApiStatus = (): ApiStatus => status;

function setStatus(next: ApiStatus): void {
  status = next;
  host?.onStatus(status);
}

function inUse(address: string): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = connect(address);
    socket.once("connect", () => {
      socket.destroy();
      resolve(true);
    });
    socket.once("error", () => resolve(false));
  });
}

export async function setApiEnabled(enabled: boolean): Promise<void> {
  const address = apiAddress();
  if (!enabled) {
    await stopServer();
    setStatus({ enabled: false, listening: false, address });
    return;
  }
  if (server) return;
  if (await inUse(address)) {
    setStatus({
      enabled: true,
      listening: false,
      address,
      error: "Another copy of Demoflow is already using this address.",
    });
    return;
  }
  if (process.platform !== "win32" && existsSync(address)) await rm(address, { force: true });
  const next = createServer((req, res) => void route(req, res));
  try {
    await new Promise<void>((resolve, reject) => {
      next.once("error", reject);
      next.listen(address, () => {
        next.off("error", reject);
        resolve();
      });
    });
    if (process.platform !== "win32") await chmod(address, 0o600);
    server = next;
    setStatus({ enabled: true, listening: true, address });
  } catch (e) {
    next.close();
    setStatus({
      enabled: true,
      listening: false,
      address,
      error: e instanceof Error ? e.message : String(e),
    });
  }
}

async function stopServer(): Promise<void> {
  const current = server;
  server = null;
  for (const stream of streams) stream.end();
  streams.clear();
  if (!current) return;
  await new Promise<void>((resolve) => current.close(() => resolve()));
  current.closeAllConnections();
  if (process.platform !== "win32") await rm(apiAddress(), { force: true });
}

export function stopApi(): void {
  void stopServer();
}

export function apiRunEvent(event: RunEvent): void {
  for (const stream of streams) stream.write(`data: ${JSON.stringify(event)}\n\n`);
}

function askEditor(request: ApiEditorRequest): Promise<ApiEditorReply> {
  const win = host?.editor();
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
      resolve(answer);
    });
    win.webContents.send(IPC.apiRequest, id, request);
  });
}

function send(res: ServerResponse, code: number, payload: unknown): void {
  const text = JSON.stringify(payload, null, 2);
  res.writeHead(code, {
    "content-type": "application/json; charset=utf-8",
    "content-length": Buffer.byteLength(text),
  });
  res.end(text);
}

function reply(res: ServerResponse, answer: ApiEditorReply): void {
  if (answer.ok) send(res, 200, answer.result);
  else
    send(res, answer.status, {
      error: answer.error,
      ...(answer.problems ? { problems: answer.problems } : {}),
    });
}

class BadRequest extends Error {}

async function readBody(req: IncomingMessage, required: boolean): Promise<unknown> {
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of req) {
    size += (chunk as Buffer).length;
    if (size > MAX_BODY) throw new BadRequest("The request body is too large.");
    chunks.push(chunk as Buffer);
  }
  const text = Buffer.concat(chunks).toString("utf8").trim();
  if (!text) {
    if (required) throw new BadRequest("This request needs a JSON body.");
    return {};
  }
  try {
    return JSON.parse(text);
  } catch {
    throw new BadRequest("The request body isn't valid JSON.");
  }
}

const isObject = (v: unknown): v is Record<string, unknown> =>
  !!v && typeof v === "object" && !Array.isArray(v);

function objectBody(v: unknown): Record<string, unknown> {
  if (!isObject(v)) throw new BadRequest("The request body must be a JSON object.");
  return v;
}

async function route(req: IncomingMessage, res: ServerResponse): Promise<void> {
  try {
    await handle(req, res);
  } catch (e) {
    if (e instanceof BadRequest) send(res, 400, { error: e.message });
    else send(res, 500, { error: e instanceof Error ? e.message : String(e) });
  }
}

async function handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
  const method = req.method ?? "GET";
  const url = new URL(req.url ?? "/", "http://localhost");
  const parts = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  const at = (m: string, ...segments: Array<string | null>) =>
    method === m &&
    parts.length === segments.length &&
    segments.every((s, i) => s === null || s === parts[i]);

  if (at("GET", "openapi.json")) return send(res, 200, openApiDocument(app.getVersion()));
  if (at("GET", "status")) {
    const project = await askEditor({ op: "read" });
    const read = project.ok
      ? (project.result as { dir: string; workflow: { name: string }; running: boolean })
      : null;
    return send(res, 200, {
      app: "Demoflow",
      version: app.getVersion(),
      project: read ? { dir: read.dir, name: read.workflow.name } : null,
      running: read?.running ?? false,
    });
  }
  if (at("GET", "nodes")) return send(res, 200, nodeCatalog(false));
  if (at("GET", "nodes", null)) {
    const entry = catalogEntry(parts[1]);
    return entry ? send(res, 200, entry) : send(res, 404, { error: "No such node type." });
  }
  if (at("POST", "validate")) {
    const result = checkWorkflow(await readBody(req, true));
    return send(res, 200, { ok: result.ok, problems: result.problems });
  }
  if (at("POST", "open")) {
    const { dir } = objectBody(await readBody(req, true));
    if (typeof dir !== "string" || !path.isAbsolute(dir))
      throw new BadRequest("dir must be an absolute folder path.");
    if (!existsSync(path.join(dir, WORKFLOW_FILE)))
      return send(res, 404, { error: `There's no Demoflow project in ${dir}.` });
    return reply(res, await askEditor({ op: "open", dir }));
  }
  if (at("GET", "project")) return reply(res, await askEditor({ op: "read" }));
  if (at("PUT", "project")) {
    const result = checkWorkflow(await readBody(req, true));
    if (!result.workflow)
      return send(res, 400, { error: "That isn't a valid project.", problems: result.problems });
    return reply(res, await askEditor({ op: "replace", workflow: result.workflow }));
  }
  if (at("POST", "project", "nodes")) {
    const b = objectBody(await readBody(req, true));
    if (typeof b.type !== "string") throw new BadRequest("type is required.");
    return reply(
      res,
      await askEditor({
        op: "addNode",
        type: b.type,
        id: typeof b.id === "string" ? b.id : undefined,
        label: typeof b.label === "string" ? b.label : undefined,
        position: position(b.position),
        params: isObject(b.params) ? b.params : undefined,
      }),
    );
  }
  if (at("PATCH", "project", "nodes", null)) {
    const b = objectBody(await readBody(req, true));
    return reply(
      res,
      await askEditor({
        op: "updateNode",
        id: parts[2],
        label: typeof b.label === "string" || b.label === null ? b.label : undefined,
        position: position(b.position),
        disabled: typeof b.disabled === "boolean" ? b.disabled : undefined,
        params: isObject(b.params) ? b.params : undefined,
      }),
    );
  }
  if (at("DELETE", "project", "nodes", null))
    return reply(res, await askEditor({ op: "removeNodes", ids: [parts[2]] }));
  if (at("POST", "project", "edges")) {
    const b = objectBody(await readBody(req, true));
    const keys = ["source", "sourceHandle", "target", "targetHandle"] as const;
    for (const k of keys) if (typeof b[k] !== "string") throw new BadRequest(`${k} is required.`);
    return reply(
      res,
      await askEditor({
        op: "addEdge",
        source: b.source as string,
        sourceHandle: b.sourceHandle as string,
        target: b.target as string,
        targetHandle: b.targetHandle as string,
      }),
    );
  }
  if (at("DELETE", "project", "edges", null))
    return reply(res, await askEditor({ op: "removeEdges", ids: [parts[2]] }));
  if (at("PATCH", "project", "settings"))
    return reply(
      res,
      await askEditor({ op: "setSettings", settings: objectBody(await readBody(req, true)) }),
    );
  if (at("PUT", "project", "variables")) {
    const b = objectBody(await readBody(req, true));
    if (Object.values(b).some((v) => typeof v !== "string"))
      throw new BadRequest("Every variable must be text.");
    return reply(
      res,
      await askEditor({ op: "setVariables", variables: b as Record<string, string> }),
    );
  }
  if (at("GET", "project", "problems")) return reply(res, await askEditor({ op: "problems" }));
  for (const op of ["undo", "redo", "save"] as const)
    if (at("POST", "project", op)) return reply(res, await askEditor({ op }));
  if (at("GET", "project", "run")) return reply(res, await askEditor({ op: "read" }).then(runOf));
  if (at("POST", "project", "run")) {
    const b = objectBody(await readBody(req, false));
    const targets = Array.isArray(b.targetNodeIds)
      ? b.targetNodeIds.filter((v): v is string => typeof v === "string")
      : undefined;
    return reply(
      res,
      await askEditor({
        op: "run",
        targetNodeIds: targets,
        fromNodeId: typeof b.fromNodeId === "string" ? b.fromNodeId : undefined,
      }),
    );
  }
  if (at("POST", "project", "run", "stop")) return reply(res, await askEditor({ op: "stop" }));
  if (at("GET", "project", "run", "events")) {
    res.writeHead(200, {
      "content-type": "text/event-stream",
      "cache-control": "no-cache",
      connection: "keep-alive",
    });
    res.write(": following run events\n\n");
    streams.add(res);
    req.on("close", () => streams.delete(res));
    return;
  }
  send(res, 404, { error: `No endpoint for ${method} ${url.pathname}. See GET /openapi.json.` });
}

function runOf(read: ApiEditorReply): ApiEditorReply {
  if (!read.ok) return read;
  const { running, runId } = read.result as { running: boolean; runId: string | null };
  return { ok: true, result: { running, runId } };
}

function position(v: unknown): { x: number; y: number } | undefined {
  if (!isObject(v)) return undefined;
  const { x, y } = v;
  if (typeof x !== "number" || typeof y !== "number")
    throw new BadRequest("position needs numbers x and y.");
  return { x, y };
}
