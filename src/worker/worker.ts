import { randomUUID } from "node:crypto";
import type { MessagePortMain } from "electron";
import { installEngine } from "../engine/install";
import type { LiveMessage } from "../shared/live";
import type { LoginRequest, MainToWorker, WorkerToMain } from "../shared/worker-protocol";
import { runWorkflow } from "./engine/runner";

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

function settle(id: string, value?: string, error?: string) {
  const entry = pending.get(id);
  pending.delete(id);
  if (error) entry?.reject(new Error(error));
  else entry?.resolve(value ?? null);
}

const sendLive = (message: LiveMessage) => live?.postMessage(message);

async function handle(msg: MainToWorker): Promise<void> {
  switch (msg.type) {
    case "cancel":
      controller.abort();
      return;
    case "secret:response":
    case "login:response":
      settle(msg.id, msg.value, msg.error);
      return;
    case "run": {
      const projectDir = msg.request.projectDir;
      try {
        await runWorkflow(msg.runId, msg.request, {
          env: msg.env,
          signal: controller.signal,
          emit: (event) => send({ type: "event", event }),
          live: live ? sendLive : null,
          login: async (req: LoginRequest) => {
            const state = await request({ type: "login:request", id: randomUUID(), request: req });
            if (!state) throw new Error("The sign-in window closed before you signed in.");
            return state;
          },
          secrets: {
            read: (name) =>
              request({ type: "secret:request", id: randomUUID(), op: "read", projectDir, name }),
            write: async (name, value) => {
              await request({
                type: "secret:request",
                id: randomUUID(),
                op: "write",
                projectDir,
                name,
                value,
              });
            },
          },
        });
      } catch (error) {
        send({
          type: "event",
          event: {
            type: "run:done",
            runId: msg.runId,
            ok: false,
            cancelled: controller.signal.aborted,
            error: error instanceof Error ? error.message : String(error),
            outputs: [],
            at: Date.now(),
          },
        });
      }
      finish();
      return;
    }
    case "install-engine": {
      try {
        await installEngine(
          msg.env.engineDir,
          (part, progress, message) =>
            send({ type: "install:progress", event: { part, progress, message } }),
          controller.signal,
        );
        send({ type: "install:done" });
      } catch (error) {
        send({
          type: "install:done",
          error: error instanceof Error ? error.message : String(error),
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
