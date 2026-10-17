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
import type { MainToWorker, WorkerEnv, WorkerToMain } from "../shared/worker-protocol";
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

export function spawnWorker(withLive: boolean, kind: WorkerKind, id: string = randomUUID()) {
  const control = new MessageChannelMain();
  const live = withLive ? new MessageChannelMain() : null;
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

function respond(
  port: MessagePortMain,
  type: "secret:response" | "login:response",
  id: string,
  work: Promise<string | null>,
): void {
  work.then(
    (value) => port.postMessage({ type, id, value: value ?? undefined } satisfies MainToWorker),
    (e: unknown) => port.postMessage({ type, id, error: errorMessage(e) } satisfies MainToWorker),
  );
}

function answerRequests(
  port: MessagePortMain,
  msg: WorkerToMain,
  window: () => BrowserWindow | null,
): boolean {
  if (msg.type === "secret:request") {
    const work =
      msg.op === "read"
        ? readSecret(msg.projectDir, msg.name)
        : writeSecret(msg.projectDir, msg.name, msg.value ?? "").then(() => "");
    respond(port, "secret:response", msg.id, work);
    return true;
  }
  if (msg.type === "login:request") {
    respond(port, "login:response", msg.id, loginInWindow(window(), msg.request));
    return true;
  }
  return false;
}

type ActiveRun = { runId: string; child: UtilityProcess; control: MessagePortMain };
let active: ActiveRun | null = null;

export const activeRunId = () => active?.runId ?? null;

export async function startRun(
  runId: string,
  request: RunRequest,
  window: () => BrowserWindow | null,
  onEvent: (e: RunEvent) => void,
  onLivePort: (port: MessagePortMain) => void,
): Promise<void> {
  if (active) throw new Error("A run is already in progress.");
  const env = await workerEnv();
  const { child, control, live } = spawnWorker(true, "run", runId);
  if (live) onLivePort(live);
  active = { runId, child, control };
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
  control.postMessage({ type: "run", env, request, runId } satisfies MainToWorker);
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

export function completeLicences(): Promise<string> {
  const root = app.isPackaged ? process.resourcesPath : path.join(app.getAppPath(), "resources");
  return completeEngineLicences(engineDir(), shippedCreditsCandidates([root])).catch(
    () => "missing",
  );
}

export async function installEngine(onEvent: (e: EngineInstallEvent) => void): Promise<void> {
  const env = await workerEnv();
  await new Promise<void>((resolve, reject) => {
    const { child, control } = spawnWorker(false, "setup");
    onWorkerMessage(control, (msg) => {
      if (msg.type === "install:progress") onEvent(msg.event);
      if (msg.type === "install:done") {
        onEvent({
          part: "headless-shell",
          step: "done",
          progress: msg.error ? null : 1,
          message: msg.error ?? "Engine ready",
          done: true,
          error: msg.error,
        });
        if (msg.error) reject(new Error(msg.error));
        else void completeLicences().then(() => resolve());
      }
    });
    child.on("exit", (code) => {
      if (code !== 0) reject(new Error(`Setting up the engine stopped unexpectedly (${code}).`));
    });
    control.postMessage({ type: "install-engine", env } satisfies MainToWorker);
  });
}
