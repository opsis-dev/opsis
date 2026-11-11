import { existsSync, readFileSync } from "node:fs";
import { request } from "node:http";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { findById } from "@opsis/video/common/lists";

type Answer = { status: number; body: unknown };
type Edge = {
  id: string;
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
};
type Node = { id: string; type: string; label?: string };
type Workflow = {
  nodes: Node[];
  edges: Edge[];
  variables?: Record<string, string>;
  settings?: { fps?: number };
};
type Project = { workflow: Workflow; revision: number; dir: string | null };
type RunEvent = { type?: string; ok?: boolean; cancelled?: boolean };
type ScriptAnswer = {
  ok?: boolean;
  value?: unknown;
  rolledBack?: boolean;
  failure?: { line?: number };
};

export type ApiTarget = { socket: string; projects: string; page: string };

const SOCKET_WAIT_MS = 60_000;
const RUN_MS = 180_000;
const POLL_MS = 250;
const ADDED_LABEL = "API click";
const SCRIPTED_LABEL = "Scripted wait";
const ROLLED_BACK_LABEL = "Undone wait";
const CALLED_ROUTES = [
  "/status",
  "/nodes",
  "/nodes/{type}",
  "/validate",
  "/open",
  "/project",
  "/project/nodes",
  "/project/nodes/{id}",
  "/project/edges",
  "/project/edges/{id}",
  "/project/settings",
  "/project/variables",
  "/project/problems",
  "/project/undo",
  "/project/redo",
  "/project/save",
  "/project/run",
  "/project/run/stop",
  "/project/run/events",
  "/project/script",
  "/project/script/stop",
];

function call(socket: string, method: string, route: string, body?: unknown): Promise<Answer> {
  return new Promise((resolve, reject) => {
    const sent = body === undefined ? undefined : JSON.stringify(body);
    const req = request(
      {
        socketPath: socket,
        method,
        path: route,
        headers: { host: "localhost", "content-type": "application/json" },
      },
      (res) => {
        let text = "";
        res.setEncoding("utf8");
        res.on("data", (chunk: string) => (text += chunk));
        res.on("end", () =>
          resolve({
            status: res.statusCode ?? 0,
            body: text ? (JSON.parse(text) as unknown) : null,
          }),
        );
      },
    );
    req.on("error", reject);
    req.end(sent);
  });
}

function follow(socket: string, until: (event: RunEvent) => boolean): Promise<RunEvent> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      req.destroy();
      reject(new Error("the run events stopped coming"));
    }, RUN_MS);
    const req = request(
      {
        socketPath: socket,
        method: "GET",
        path: "/project/run/events",
        headers: { host: "localhost" },
      },
      (res) => {
        let buffered = "";
        res.setEncoding("utf8");
        res.on("data", (chunk: string) => {
          buffered += chunk;
          const blocks = buffered.split("\n\n");
          buffered = blocks.pop() ?? "";
          for (const block of blocks) {
            const data = block.split("\n").find((line) => line.startsWith("data: "));
            if (data === undefined) continue;
            const event = JSON.parse(data.slice(6)) as RunEvent;
            if (until(event)) {
              clearTimeout(timer);
              req.destroy();
              resolve(event);
            }
          }
        });
      },
    );
    req.on("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    req.end();
  });
}

async function project(socket: string): Promise<Project> {
  return (await call(socket, "GET", "/project")).body as Project;
}

function okOf(answer: Answer): boolean | undefined {
  return (answer.body as { ok?: boolean }).ok;
}

function expect(ok: boolean, problem: string): string {
  return ok ? "" : problem;
}

function edgeBetween(workflow: Workflow, edge: Edge): boolean {
  return workflow.edges.some(
    (each) =>
      each.source === edge.source &&
      each.sourceHandle === edge.sourceHandle &&
      each.target === edge.target &&
      each.targetHandle === edge.targetHandle,
  );
}

async function untilSocket(socket: string): Promise<boolean> {
  const deadline = Date.now() + SOCKET_WAIT_MS;
  while (Date.now() < deadline) {
    if (existsSync(socket)) {
      const answer = await call(socket, "GET", "/status").catch(() => null);
      if (answer?.status === 200) return true;
    }
    await sleep(POLL_MS);
  }
  return false;
}

function checks(target: ApiTarget): [string, () => Promise<string>][] {
  const { socket, page } = target;
  const dir = path.join(target.projects, "todomvc");
  let removed: Edge | null = null;
  let added = "";
  return [
    [
      "api-status-answers",
      async () => {
        const answer = await call(socket, "GET", "/status");
        const status = answer.body as { app?: string; version?: string };
        return expect(
          answer.status === 200 && Boolean(status.app) && Boolean(status.version),
          `the status was ${answer.status} ${JSON.stringify(answer.body)}`,
        );
      },
    ],
    [
      "api-documents-its-routes",
      async () => {
        const answer = await call(socket, "GET", "/openapi.json");
        const paths = Object.keys((answer.body as { paths?: object }).paths ?? {});
        const missing = CALLED_ROUTES.filter((route) => !paths.includes(route));
        return expect(missing.length === 0, `undocumented: ${missing.join(", ")}`);
      },
    ],
    [
      "api-lists-the-node-types",
      async () => {
        const all = await call(socket, "GET", "/nodes");
        const one = await call(socket, "GET", "/nodes/browser.click");
        const none = await call(socket, "GET", "/nodes/browser.nothing");
        const types = (all.body as { type: string }[]).map((node) => node.type);
        return (
          expect(types.includes("browser.click"), "browser.click isn't listed") +
          expect(one.status === 200, ` one type answered ${one.status}`) +
          expect(none.status === 404, ` an unknown type answered ${none.status}`)
        );
      },
    ],
    [
      "api-opens-a-project",
      async () => {
        const opened = await call(socket, "POST", "/open", { dir });
        const status = (await call(socket, "GET", "/status")).body as {
          project?: { dir?: string } | null;
        };
        return expect(
          opened.status === 200 && status.project?.dir === dir,
          `opening answered ${opened.status}; the status shows ${JSON.stringify(status.project)}`,
        );
      },
    ],
    [
      "api-sets-the-variables",
      async () => {
        const answer = await call(socket, "PUT", "/project/variables", { "Site address": page });
        const shown = (await project(socket)).workflow.variables?.["Site address"];
        return expect(answer.status === 200 && shown === page, `the site address is ${shown}`);
      },
    ],
    [
      "api-adds-and-renames-a-node",
      async () => {
        const answer = await call(socket, "POST", "/project/nodes", {
          type: "browser.click",
          label: "API added",
          params: { target: "Save draft" },
          position: { x: 0, y: 900 },
        });
        added = (answer.body as { id?: string }).id ?? "";
        const renamed = await call(socket, "PATCH", `/project/nodes/${encodeURIComponent(added)}`, {
          label: ADDED_LABEL,
        });
        const node = findById((await project(socket)).workflow.nodes, added);
        return expect(
          answer.status === 200 && renamed.status === 200 && node?.label === ADDED_LABEL,
          `adding answered ${answer.status}, renaming ${renamed.status}; the node is ${JSON.stringify(node)}`,
        );
      },
    ],
    [
      "api-explains-a-bad-request",
      async () => {
        const answer = await call(socket, "POST", "/project/nodes", {});
        const missing = await call(socket, "DELETE", "/project/nodes/nothing-here");
        const error = (answer.body as { error?: string }).error;
        return (
          expect(
            answer.status === 400 && error === "type: is required.",
            `a node without a type: ${error}`,
          ) + expect(missing.status === 404, ` deleting a missing node answered ${missing.status}`)
        );
      },
    ],
    [
      "api-removes-and-connects-an-edge",
      async () => {
        const [first] = (await project(socket)).workflow.edges;
        if (first === undefined) return "the project has no connections";
        removed = first;
        const gone = await call(socket, "DELETE", `/project/edges/${encodeURIComponent(first.id)}`);
        const without = !edgeBetween((await project(socket)).workflow, first);
        const joined = await call(socket, "POST", "/project/edges", {
          source: first.source,
          sourceHandle: first.sourceHandle,
          target: first.target,
          targetHandle: first.targetHandle,
        });
        const back = edgeBetween((await project(socket)).workflow, first);
        return expect(
          gone.status === 200 && without && joined.status === 200 && back,
          `removing answered ${gone.status}, connecting ${joined.status}`,
        );
      },
    ],
    [
      "api-undoes-and-redoes",
      async () => {
        if (removed === null) return "no connection to undo";
        const undone = await call(socket, "POST", "/project/undo");
        const afterUndo = edgeBetween((await project(socket)).workflow, removed);
        const redone = await call(socket, "POST", "/project/redo");
        const afterRedo = edgeBetween((await project(socket)).workflow, removed);
        return expect(
          undone.status === 200 && !afterUndo && redone.status === 200 && afterRedo,
          `undo left the connection ${afterUndo ? "in" : "out"}, redo ${afterRedo ? "in" : "out"}`,
        );
      },
    ],
    [
      "api-changes-the-settings",
      async () => {
        const wrong = await call(socket, "PATCH", "/project/settings", { colour: "red" });
        const changed = await call(socket, "PATCH", "/project/settings", { fps: 60 });
        const fps = (await project(socket)).workflow.settings?.fps;
        const back = await call(socket, "PATCH", "/project/settings", { fps: 30 });
        return expect(
          wrong.status === 400 && changed.status === 200 && fps === 60 && back.status === 200,
          `an unknown setting answered ${wrong.status}; the frame rate became ${fps}`,
        );
      },
    ],
    [
      "api-saves-the-project",
      async () => {
        const saved = await call(socket, "POST", "/project/save");
        const file = readFileSync(path.join(dir, "project.opsis.json"), "utf8");
        return expect(
          saved.status === 200 && file.includes(ADDED_LABEL),
          `saving answered ${saved.status}; the file ${file.includes(ADDED_LABEL) ? "has" : "lacks"} the added node`,
        );
      },
    ],
    [
      "api-runs-a-script-that-adds-a-node",
      async () => {
        const answer = await call(socket, "POST", "/project/script", {
          source: `local node = opsis.nodes.add { type = "browser.wait", label = "${SCRIPTED_LABEL}" }\nreturn node.id`,
        });
        const outcome = answer.body as ScriptAnswer;
        const node = findById((await project(socket)).workflow.nodes, String(outcome.value));
        const status = await call(socket, "GET", "/project/script");
        return expect(
          answer.status === 200 &&
            outcome.ok === true &&
            node?.label === SCRIPTED_LABEL &&
            status.status === 200,
          `the script answered ${answer.status} ${JSON.stringify(answer.body)}`,
        );
      },
    ],
    [
      "api-rolls-back-a-script-that-fails",
      async () => {
        const answer = await call(socket, "POST", "/project/script", {
          source: `opsis.nodes.add { type = "browser.wait", label = "${ROLLED_BACK_LABEL}" }\nerror("broken")`,
        });
        const outcome = answer.body as ScriptAnswer;
        const left = (await project(socket)).workflow.nodes.some(
          (node) => node.label === ROLLED_BACK_LABEL,
        );
        const stopped = await call(socket, "POST", "/project/script/stop");
        return expect(
          answer.status === 200 &&
            outcome.ok === false &&
            outcome.rolledBack === true &&
            outcome.failure?.line === 2 &&
            !left &&
            stopped.status === 200,
          `the failing script answered ${answer.status} ${JSON.stringify(answer.body)}; its node ${left ? "stayed" : "is gone"}`,
        );
      },
    ],
    [
      "api-reports-problems",
      async () => {
        const answer = await call(socket, "GET", "/project/problems");
        const check = answer.body as { ok?: unknown; problems?: unknown };
        return expect(
          answer.status === 200 && typeof check.ok === "boolean" && Array.isArray(check.problems),
          `the problems answered ${answer.status} ${JSON.stringify(answer.body)}`,
        );
      },
    ],
    [
      "api-validates-without-opening",
      async () => {
        const { workflow } = await project(socket);
        const good = await call(socket, "POST", "/validate", workflow);
        const broken = {
          ...workflow,
          nodes: [
            ...workflow.nodes,
            { ...workflow.nodes[0], id: "broken", type: "browser.nothing" },
          ],
        };
        const bad = await call(socket, "POST", "/validate", broken);
        return expect(
          good.status === 200 && bad.status === 200 && okOf(bad) === false,
          `validating answered ${good.status} and ${bad.status}; the broken flow was ok: ${okOf(bad)}`,
        );
      },
    ],
    [
      "api-runs-a-take",
      async () => {
        const done = follow(socket, (event) => event.type === "run:done");
        const started = await call(socket, "POST", "/project/run", { targetNodeIds: ["record"] });
        const end = await done;
        return expect(
          started.status === 200 && end.ok === true && end.cancelled === false,
          `the run answered ${started.status} and ended ${JSON.stringify(end)}`,
        );
      },
    ],
    [
      "api-stops-a-run",
      async () => {
        const going = follow(socket, (event) => event.type === "node:start");
        const done = follow(socket, (event) => event.type === "run:done");
        const started = await call(socket, "POST", "/project/run", { fromNodeId: "session" });
        await going;
        const stopped = await call(socket, "POST", "/project/run/stop");
        const end = await done;
        return expect(
          started.status === 200 && stopped.status === 200 && end.cancelled === true,
          `stopping answered ${stopped.status} and the run ended ${JSON.stringify(end)}`,
        );
      },
    ],
  ];
}

export async function runApiChecks(
  target: ApiTarget,
  log: (line: string) => void,
): Promise<boolean> {
  if (!(await untilSocket(target.socket))) {
    log("check api-socket-opens: FAILED the app never answered on its socket");
    return false;
  }
  let passed = true;
  for (const [name, check] of checks(target)) {
    const problem = await check().catch((error: unknown) => String(error));
    passed = passed && problem === "";
    log(`check ${name}: ${problem === "" ? "ok" : `FAILED ${problem.trim()}`}`);
  }
  return passed;
}
