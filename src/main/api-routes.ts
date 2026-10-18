import { existsSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { app } from "electron";
import type { z } from "zod";
import {
  addEdgeBody,
  addNodeBody,
  objectBody,
  openBody,
  runBody,
  updateNodeBody,
  variablesBody,
} from "../shared/api/bodies";
import { catalogEntry, nodeCatalog } from "../shared/api/catalog";
import { checkWorkflow } from "../shared/api/check";
import { openApiDocument } from "../shared/api/openapi";
import {
  apiFailure,
  apiOk,
  type ApiProjectState,
  type ApiReply,
  type ApiRunState,
} from "../shared/api/types";
import { errorMessage } from "../shared/errors";
import { WORKFLOW_FILE } from "../shared/schema/workflow";
import { askEditor, followRunEvents, readEditorProject } from "./api-editor";

const MAX_BODY = 20 * 1024 * 1024;

type Context = { req: IncomingMessage; params: string[] };
type Answer = ApiReply<unknown>;
type Handler =
  | { kind: "answer"; run: (context: Context) => Answer | Promise<Answer> }
  | { kind: "stream"; stream: (req: IncomingMessage, res: ServerResponse) => void };
type Route = Handler & { method: string; segments: string[] };

class BadRequest extends Error {}

function send(res: ServerResponse, code: number, payload: unknown): void {
  const text = JSON.stringify(payload, null, 2);
  res.writeHead(code, {
    "content-type": "application/json; charset=utf-8",
    "content-length": Buffer.byteLength(text),
  });
  res.end(text);
}

function reply(res: ServerResponse, answer: ApiReply<unknown>): void {
  if (answer.ok) send(res, 200, answer.result);
  else
    send(res, answer.status, {
      error: answer.error,
      ...(answer.problems ? { problems: answer.problems } : {}),
    });
}

async function readBody(req: IncomingMessage, required: boolean): Promise<unknown> {
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of req) {
    const bytes = Buffer.from(chunk);
    size += bytes.length;
    if (size > MAX_BODY) throw new BadRequest("The request body is too large.");
    chunks.push(bytes);
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

async function parseBody<T>(
  req: IncomingMessage,
  schema: z.ZodType<T>,
  required = true,
): Promise<T> {
  const parsed = schema.safeParse(await readBody(req, required));
  if (!parsed.success) throw new BadRequest(parsed.error.issues[0].message);
  return parsed.data;
}

async function appStatus(): Promise<Answer> {
  const project = await readEditorProject();
  const read = project.ok ? project.result : null;
  return apiOk({
    app: "Demoflow",
    version: app.getVersion(),
    project: read ? { dir: read.dir, name: read.workflow.name } : null,
    running: read?.running ?? false,
  });
}

async function openInEditor({ req }: Context): Promise<Answer> {
  const { dir } = await parseBody(req, openBody);
  if (!path.isAbsolute(dir)) throw new BadRequest("dir must be an absolute folder path.");
  if (!existsSync(path.join(dir, WORKFLOW_FILE)))
    return apiFailure(404, `There's no Demoflow project in ${dir}.`);
  return askEditor({ op: "open", dir });
}

const checkedBody = async (req: IncomingMessage) => checkWorkflow(await readBody(req, true));

async function validate({ req }: Context): Promise<Answer> {
  const { ok, problems } = await checkedBody(req);
  return apiOk({ ok, problems });
}

async function replaceProject({ req }: Context): Promise<Answer> {
  const result = await checkedBody(req);
  if (!result.workflow)
    return { ...apiFailure(400, "That isn't a valid project."), problems: result.problems };
  return askEditor({ op: "replace", workflow: result.workflow });
}

const runState = (read: ApiReply<ApiProjectState>): ApiReply<ApiRunState> =>
  read.ok ? apiOk({ running: read.result.running, runId: read.result.runId }) : read;

const segmentsOf = (pattern: string) => pattern.split("/").filter(Boolean);

const route = (
  method: string,
  pattern: string,
  run: (context: Context) => Answer | Promise<Answer>,
): Route => ({ kind: "answer", method, segments: segmentsOf(pattern), run });

const streamRoute = (
  method: string,
  pattern: string,
  stream: (req: IncomingMessage, res: ServerResponse) => void,
): Route => ({ kind: "stream", method, segments: segmentsOf(pattern), stream });

const ROUTES: Route[] = [
  route("GET", "/openapi.json", () => apiOk(openApiDocument(app.getVersion()))),
  route("GET", "/status", appStatus),
  route("GET", "/nodes", () => apiOk(nodeCatalog())),
  route("GET", "/nodes/:type", ({ params: [type] }) => {
    const entry = catalogEntry(type);
    return entry ? apiOk(entry) : apiFailure(404, "No such node type.");
  }),
  route("POST", "/validate", validate),
  route("POST", "/open", openInEditor),
  route("GET", "/project", () => askEditor({ op: "read" })),
  route("PUT", "/project", replaceProject),
  route("POST", "/project/nodes", async ({ req }) =>
    askEditor({ op: "addNode", ...(await parseBody(req, addNodeBody)) }),
  ),
  route("PATCH", "/project/nodes/:id", async ({ req, params: [id] }) =>
    askEditor({ op: "updateNode", ...(await parseBody(req, updateNodeBody)), id }),
  ),
  route("DELETE", "/project/nodes/:id", ({ params: [id] }) =>
    askEditor({ op: "removeNodes", ids: [id] }),
  ),
  route("POST", "/project/edges", async ({ req }) =>
    askEditor({ op: "addEdge", ...(await parseBody(req, addEdgeBody)) }),
  ),
  route("DELETE", "/project/edges/:id", ({ params: [id] }) =>
    askEditor({ op: "removeEdges", ids: [id] }),
  ),
  route("PATCH", "/project/settings", async ({ req }) =>
    askEditor({ op: "setSettings", settings: await parseBody(req, objectBody) }),
  ),
  route("PUT", "/project/variables", async ({ req }) =>
    askEditor({ op: "setVariables", variables: await parseBody(req, variablesBody) }),
  ),
  route("GET", "/project/problems", () => askEditor({ op: "problems" })),
  route("POST", "/project/undo", () => askEditor({ op: "undo" })),
  route("POST", "/project/redo", () => askEditor({ op: "redo" })),
  route("POST", "/project/save", () => askEditor({ op: "save" })),
  route("GET", "/project/run", async () => runState(await readEditorProject())),
  route("POST", "/project/run", async ({ req }) =>
    askEditor({ op: "run", ...(await parseBody(req, runBody, false)) }),
  ),
  route("POST", "/project/run/stop", () => askEditor({ op: "stop" })),
  streamRoute("GET", "/project/run/events", followRunEvents),
];

function pathParts(pathname: string): string[] {
  try {
    return segmentsOf(pathname).map(decodeURIComponent);
  } catch {
    throw new BadRequest("The request address isn't encoded correctly.");
  }
}

function findRoute(method: string, parts: string[]) {
  const found = ROUTES.find(
    (r) =>
      r.method === method &&
      r.segments.length === parts.length &&
      r.segments.every((s, i) => s.startsWith(":") || s === parts[i]),
  );
  return found && { found, params: parts.filter((_, i) => found.segments[i].startsWith(":")) };
}

export async function handleApiRequest(req: IncomingMessage, res: ServerResponse): Promise<void> {
  try {
    const method = req.method ?? "GET";
    const url = new URL(req.url ?? "/", "http://localhost");
    const match = findRoute(method, pathParts(url.pathname));
    if (!match) {
      send(res, 404, {
        error: `No endpoint for ${method} ${url.pathname}. See GET /openapi.json.`,
      });
      return;
    }
    const { found, params } = match;
    if (found.kind === "stream") found.stream(req, res);
    else reply(res, await found.run({ req, params }));
  } catch (e) {
    if (e instanceof BadRequest) send(res, 400, { error: e.message });
    else send(res, 500, { error: errorMessage(e) });
  }
}
