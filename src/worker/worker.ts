import { randomUUID } from "node:crypto";
import type { MessagePortMain } from "electron";
import { installEngine } from "../engine/install";
import { errorMessage } from "../shared/errors";
import type { LiveMessage } from "../shared/live";
import { pendingReplies } from "../shared/pending-replies";
import { runFailed } from "../shared/schema/events";
import type {
  LoginRequest,
  MainMessage,
  MainToWorker,
  WorkerToMain,
} from "../shared/worker-protocol";
import type { ExecutorContext } from "./engine/context";
import { runWorkflow } from "./engine/runner";
import { runAnalysis } from "./media/run-analysis";
import { renderCardStill } from "./render/card-still";

let control: MessagePortMain | null = null;
let live: MessagePortMain | null = null;
const controller = new AbortController();
const replies = pendingReplies<{ value: string | null } | { error: string }>();

const send = (msg: WorkerToMain) => control?.postMessage(msg);

async function request(msg: Extract<WorkerToMain, { id: string }>): Promise<string | null> {
  const reply = replies.wait(msg.id);
  send(msg);
  const outcome = await reply;
  if ("error" in outcome) throw new Error(outcome.error);
  return outcome.value;
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

async function errorOf(work: () => Promise<unknown>): Promise<string | undefined> {
  try {
    await work();
    return undefined;
  } catch (error) {
    return errorMessage(error);
  }
}

async function run(msg: MainMessage<"run">) {
  const error = await errorOf(() =>
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

async function installEngineFor(msg: MainMessage<"install-engine">) {
  const error = await errorOf(() =>
    installEngine(
      msg.env.engineDir,
      (event) => send({ type: "install:progress", event }),
      controller.signal,
    ),
  );
  send({ type: "install:done", error });
  finish();
}

async function cardStill(msg: MainMessage<"still">) {
  const error = await errorOf(() => renderCardStill(msg, controller.signal));
  send({ type: "still:done", id: msg.id, error });
}

async function handle(msg: MainToWorker): Promise<void> {
  switch (msg.type) {
    case "cancel":
      controller.abort();
      return;
    case "secret:value":
    case "login:state":
      replies.settle(msg.id, { value: msg.value });
      return;
    case "secret:written":
      replies.settle(msg.id, { value: null });
      return;
    case "request:failed":
      replies.settle(msg.id, { error: msg.error });
      return;
    case "run":
      return run(msg);
    case "still":
      return cardStill(msg);
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
  const [controlPort, livePort] = event.ports;
  control = controlPort;
  live = livePort ?? null;
  control.on("message", (e: { data: MainToWorker }) => void handle(e.data));
  control.on("close", () => replies.settleAll({ error: "Demoflow closed before it answered." }));
  control.start();
  live?.start();
});
