import { randomUUID } from "node:crypto";
import path from "node:path";
import {
  app,
  MessageChannelMain,
  utilityProcess,
  type MessagePortMain,
  type UtilityProcess,
} from "electron";
import { resolveEngineDir } from "../engine/paths";
import { isWorkerToMain, type WorkerEnv, type WorkerToMain } from "../shared/worker-protocol";
import { readSettings } from "./settings";
import {
  makeWorkerTemp,
  removeWorkerTemp,
  tempEnv,
  workerTempName,
  type WorkerKind,
} from "./worker-temp";

export function engineDir(): string {
  return resolveEngineDir({
    isPackaged: app.isPackaged,
    resourcesPath: process.resourcesPath,
    userData: app.getPath("userData"),
  });
}

export function workerEnv(): WorkerEnv {
  return {
    appRoot: app.getAppPath(),
    isPackaged: app.isPackaged,
    resourcesPath: process.resourcesPath,
    userData: app.getPath("userData"),
    engineDir: engineDir(),
    settings: readSettings(),
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
    serviceName: "Opsis worker",
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
  port.on("message", (e) => {
    if (isWorkerToMain(e.data)) handler(e.data);
  });
}
