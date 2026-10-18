import type { MessagePortMain, UtilityProcess } from "electron";
import { errorMessage } from "../shared/errors";
import type { RunRequest } from "../shared/ipc";
import type { RunEvent } from "../shared/schema/events";
import type { MainToWorker, WorkerReply, WorkerToMain } from "../shared/worker-protocol";
import { loginInWindow } from "./browser-windows";
import { readSecret, writeSecret } from "./secrets";
import { getMainWindow } from "./window-registry";
import { onWorkerMessage, spawnWorker, workerEnv } from "./workers";

const KILL_CANCELLED_RUN_AFTER_MS = 4000;

function respond<T>(
  port: MessagePortMain,
  id: string,
  work: Promise<T>,
  reply: (value: T) => WorkerReply,
): void {
  work.then(
    (value) => port.postMessage(reply(value) satisfies MainToWorker),
    (e: unknown) =>
      port.postMessage({
        type: "request:failed",
        id,
        error: errorMessage(e),
      } satisfies MainToWorker),
  );
}

function answerRequests(port: MessagePortMain, msg: WorkerToMain): boolean {
  switch (msg.type) {
    case "secret:read":
      respond(port, msg.id, readSecret(msg.projectDir, msg.name), (value) => ({
        type: "secret:value",
        id: msg.id,
        value,
      }));
      return true;
    case "secret:write":
      respond(port, msg.id, writeSecret(msg.projectDir, msg.name, msg.value), () => ({
        type: "secret:written",
        id: msg.id,
      }));
      return true;
    case "login:request":
      respond(port, msg.id, loginInWindow(getMainWindow(), msg.request), (value) => ({
        type: "login:state",
        id: msg.id,
        value,
      }));
      return true;
    default:
      return false;
  }
}

type ActiveRun = {
  runId: string;
  projectDir: string;
  child: UtilityProcess;
  control: MessagePortMain;
};
let active: ActiveRun | null = null;

export const activeRun = (): { runId: string; projectDir: string } | null =>
  active && { runId: active.runId, projectDir: active.projectDir };

export async function startRun(
  runId: string,
  request: RunRequest,
  onEvent: (e: RunEvent) => void,
): Promise<MessagePortMain | null> {
  if (active) throw new Error("A run is already in progress.");
  const { child, control, live } = spawnWorker("run", runId);
  active = { runId, projectDir: request.projectDir, child, control };
  let finished = false;
  onWorkerMessage(control, (msg) => {
    if (answerRequests(control, msg)) return;
    if (msg.type === "event") {
      if (msg.event.type === "run:done") finished = true;
      onEvent(msg.event);
    }
  });
  child.on("exit", (code) => {
    if (active?.runId === runId) active = null;
    if (!finished)
      onEvent({
        type: "run:done",
        runId,
        ok: false,
        cancelled: code === null,
        error: code === null ? "Stopped" : `The run stopped unexpectedly (${code})`,
        outputs: [],
        at: Date.now(),
      });
  });
  try {
    const env = await workerEnv();
    control.postMessage({ type: "run", env, request, runId } satisfies MainToWorker);
    return live;
  } catch (error) {
    child.kill();
    throw error;
  }
}

export function cancelRun(): void {
  const run = active;
  if (!run) return;
  run.control.postMessage({ type: "cancel" } satisfies MainToWorker);
  setTimeout(() => {
    if (active?.runId === run.runId) run.child.kill();
  }, KILL_CANCELLED_RUN_AFTER_MS);
}

export const isRunning = () => active !== null;
