import { randomUUID } from "node:crypto";
import { existsSync } from "node:fs";
import path from "node:path";
import {
  app,
  MessageChannelMain,
  utilityProcess,
  type BrowserWindow,
  type MessagePortMain,
  type UtilityProcess,
} from "electron";
import { engineStatus, resolveEngineDir } from "../engine/paths";
import { completeEngineLicences, shippedCreditsCandidates } from "../engine/licences";
import type { EngineInstallEvent, EnvStatus, RunRequest } from "../shared/ipc";
import type { RunEvent } from "../shared/schema/events";
import type { MainToWorker, WorkerEnv, WorkerReply, WorkerToMain } from "../shared/worker-protocol";
import { loginInWindow } from "./browser-windows";
import { readSecret, writeSecret } from "./secrets";
import { readSettings } from "./settings";
import {
  makeWorkerTemp,
  removeWorkerTemp,
  tempEnv,
  workerTempName,
  type WorkerKind,
} from "./worker-temp";
import { errorMessage } from "../shared/errors";

export function engineDir(): string {
  return resolveEngineDir({
    isPackaged: app.isPackaged,
    resourcesPath: process.resourcesPath,
    userData: app.getPath("userData"),
  });
}

export async function workerEnv(): Promise<WorkerEnv> {
  return {
    appRoot: app.getAppPath(),
    isPackaged: app.isPackaged,
    resourcesPath: process.resourcesPath,
    userData: app.getPath("userData"),
    engineDir: engineDir(),
    settings: await readSettings(),
  };
}

export const workerTempRoot = () => path.join(app.getPath("userData"), "tmp");

type SpawnedWorker = {
  child: UtilityProcess;
  control: MessagePortMain;
  live: MessagePortMain | null;
};

export function spawnWorker(kind: WorkerKind, id: string = randomUUID()): SpawnedWorker {
  const control = new MessageChannelMain();
  const live = kind === "run" ? new MessageChannelMain() : null;
  const temp = makeWorkerTemp(workerTempRoot(), workerTempName(kind, id));
  const child = utilityProcess.fork(path.join(__dirname, "worker.cjs"), [], {
    serviceName: "Demoflow worker",
    stdio: "pipe",
    env: { ...process.env, ...tempEnv(temp) },
  });
  child.on("exit", () => removeWorkerTemp(temp));
  child.stdout?.on("data", (d: Buffer) => process.stdout.write(`[worker] ${d}`));
  child.stderr?.on("data", (d: Buffer) => process.stderr.write(`[worker] ${d}`));
  child.postMessage(null, live ? [control.port2, live.port2] : [control.port2]);
  control.port1.start();
  return { child, control: control.port1, live: live?.port1 ?? null };
}

export function onWorkerMessage(port: MessagePortMain, handler: (msg: WorkerToMain) => void): void {
  port.on("message", (e) => handler(e.data as WorkerToMain));
}

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

function answerRequests(
  port: MessagePortMain,
  msg: WorkerToMain,
  window: () => BrowserWindow | null,
): boolean {
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
      respond(port, msg.id, loginInWindow(window(), msg.request), (value) => ({
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
  window: () => BrowserWindow | null,
  onEvent: (e: RunEvent) => void,
  onLivePort: (port: MessagePortMain) => void,
): Promise<void> {
  if (active) throw new Error("A run is already in progress.");
  const { child, control, live } = spawnWorker("run", runId);
  active = { runId, projectDir: request.projectDir, child, control };
  if (live) onLivePort(live);
  let finished = false;
  onWorkerMessage(control, (msg) => {
    if (answerRequests(control, msg, window)) return;
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
  }, 4000);
}

export const isRunning = () => active !== null;

function edition(): EnvStatus["edition"] {
  if (!app.isPackaged) return "dev";
  return existsSync(path.join(process.resourcesPath, "engine")) ? "full" : "lite";
}

export function environment(): EnvStatus {
  return {
    platform: process.platform,
    arch: process.arch,
    isPackaged: app.isPackaged,
    edition: edition(),
    userData: app.getPath("userData"),
    versions: {
      electron: process.versions.electron,
      chrome: process.versions.chrome,
      node: process.versions.node,
      app: app.getVersion(),
    },
    engine: engineStatus(engineDir()),
  };
}

export async function completeLicences(): Promise<void> {
  const root = app.isPackaged ? process.resourcesPath : path.join(app.getAppPath(), "resources");
  try {
    const result = await completeEngineLicences(engineDir(), shippedCreditsCandidates([root]));
    if (result === "missing")
      process.stderr.write("[licences] the recording browser's licence file is missing\n");
  } catch (error) {
    process.stderr.write(`[licences] ${errorMessage(error)}\n`);
  }
}

export async function installEngine(onEvent: (e: EngineInstallEvent) => void): Promise<void> {
  const env = await workerEnv();
  await new Promise<void>((resolve, reject) => {
    const { child, control } = spawnWorker("setup");
    let installed = false;
    onWorkerMessage(control, (msg) => {
      if (msg.type === "install:progress") onEvent(msg.event);
      if (msg.type === "install:done") {
        installed = true;
        onEvent({
          part: "headless-shell",
          step: "done",
          progress: msg.error ? null : 1,
          message: msg.error ?? "Engine ready",
          done: true,
          error: msg.error,
        });
        if (msg.error) reject(new Error(msg.error));
        else void completeLicences().then(resolve);
      }
    });
    child.on("exit", (code) => {
      if (!installed) reject(new Error(`Setting up the engine stopped unexpectedly (${code}).`));
    });
    control.postMessage({ type: "install-engine", env } satisfies MainToWorker);
  });
}
