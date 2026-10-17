import { randomUUID } from "node:crypto";
import type { MessagePortMain } from "electron";
import { installEngine } from "../engine/install";
import type { LiveMessage } from "../shared/live";
import type { LoginRequest, MainToWorker, WorkerToMain } from "../shared/worker-protocol";
import { runFailed } from "../shared/run-progress";
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
    case "run": {
      try {
        await runWorkflow(msg.runId, msg.request, {
          env: msg.env,
          signal: controller.signal,
          emit: (event) => send({ type: "event", event }),
          live: live ? sendLive : null,
          login,
          secrets: secretsFor(msg.request.projectDir),
        });
      } catch (error) {
        send({
          type: "event",
          event: runFailed(msg.runId, errorMessage(error), {
            cancelled: controller.signal.aborted,
          }),
        });
      }
      finish();
      return;
    }
    case "still": {
      try {
        await renderCardStill(msg, controller.signal);
        send({ type: "still:done", id: msg.id });
      } catch (error) {
        send({
          type: "still:done",
          id: msg.id,
          error: errorMessage(error),
        });
      }
      return;
    }
    case "analysis": {
      const result = await runAnalysis(msg.env, msg.request);
      send({ type: "analysis:done", id: msg.id, result });
      return;
    }
    case "install-engine": {
      try {
        await installEngine(
          msg.env.engineDir,
          (event) => send({ type: "install:progress", event }),
          controller.signal,
        );
        send({ type: "install:done" });
      } catch (error) {
        send({
          type: "install:done",
          error: errorMessage(error),
        });
      }
      finish();
      return;
    }
  }
}

function finish() {
  setTimeout(() => process.exit(0), 50);
}

process.parentPort.once("message", (event) => {
  [control, live = null] = event.ports;
  control.on("message", (e) => void handle(e.data as MainToWorker));
  control.start();
  live?.start();
});
