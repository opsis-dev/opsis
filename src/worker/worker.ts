import { randomUUID } from "node:crypto";
import type { MessagePortMain } from "electron";
import { installEngine } from "../engine/install";
import type { LiveMessage } from "../shared/live";
import type { LoginRequest, MainToWorker, WorkerToMain } from "../shared/worker-protocol";
import { runFailed } from "../shared/schema/events";
import type { ExecutorContext } from "./engine/context";
import { runWorkflow } from "./engine/runner";
import { renderCardStill } from "./render/card-still";
import { runAnalysis } from "./media/run-analysis";
import { errorMessage } from "../shared/errors";

let control: MessagePortMain | null = null;
let live: MessagePortMain | null = null;
const controller = new AbortController();
const pending = new Map<
  string,
  { resolve: (v: string | null) => void; reject: (e: Error) => void }
>();

const send = (msg: WorkerToMain) => control?.postMessage(msg);

function request(msg: Extract<WorkerToMain, { id: string }>): Promise<string | null> {
  return new Promise((resolve, reject) => {
    pending.set(msg.id, { resolve, reject });
    send(msg);
  });
}

function settle(id: string, outcome: { value: string | null } | { error: string }) {
  const entry = pending.get(id);
  pending.delete(id);
  if ("error" in outcome) entry?.reject(new Error(outcome.error));
  else entry?.resolve(outcome.value);
}

const sendLive = (message: LiveMessage) => live?.postMessage(message);

async function login(req: LoginRequest): Promise<string> {
  const state = await request({ type: "login:request", id: randomUUID(), request: req });
  if (!state) throw new Error("The sign-in window closed before you signed in.");
  return state;
}

const secretsFor = (projectDir: string): ExecutorContext["secrets"] => ({
  read: (name) => request({ type: "secret:read", id: randomUUID(), projectDir, name }),
  write: async (name, value) => {
    await request({ type: "secret:write", id: randomUUID(), projectDir, name, value });
  },
});

const EXIT_FLUSH_MS = 50;

async function failureOf(work: () => Promise<unknown>): Promise<string | undefined> {
  try {
    await work();
    return undefined;
  } catch (error) {
    return errorMessage(error);
  }
}

const withError = (error: string | undefined) => (error === undefined ? {} : { error });

async function run(msg: Extract<MainToWorker, { type: "run" }>) {
  const error = await failureOf(() =>
    runWorkflow(msg.runId, msg.request, {
      env: msg.env,
      signal: controller.signal,
      emit: (event) => send({ type: "event", event }),
      live: live ? sendLive : null,
      login,
      secrets: secretsFor(msg.request.projectDir),
    }),
  );
  if (error !== undefined)
    send({
      type: "event",
      event: runFailed(msg.runId, error, { cancelled: controller.signal.aborted }),
    });
  finish();
}

async function installEngineFor(msg: Extract<MainToWorker, { type: "install-engine" }>) {
  const error = await failureOf(() =>
    installEngine(
      msg.env.engineDir,
      (event) => send({ type: "install:progress", event }),
      controller.signal,
    ),
  );
  send({ type: "install:done", ...withError(error) });
  finish();
}

async function handle(msg: MainToWorker): Promise<void> {
  switch (msg.type) {
    case "cancel":
      controller.abort();
      return;
    case "secret:value":
    case "login:state":
      settle(msg.id, { value: msg.value });
      return;
    case "secret:written":
      settle(msg.id, { value: null });
      return;
    case "request:failed":
      settle(msg.id, { error: msg.error });
      return;
    case "run":
      return run(msg);
    case "still": {
      const error = await failureOf(() => renderCardStill(msg, controller.signal));
      send({ type: "still:done", id: msg.id, ...withError(error) });
      return;
    }
    case "analysis":
      send({ type: "analysis:done", id: msg.id, result: await runAnalysis(msg.env, msg.request) });
      return;
    case "install-engine":
      return installEngineFor(msg);
  }
}

function finish() {
  setTimeout(() => process.exit(0), EXIT_FLUSH_MS);
}

process.parentPort.once("message", (event) => {
  [control, live = null] = event.ports;
  control.on("message", (e) => void handle(e.data as MainToWorker));
  control.start();
  live?.start();
});
