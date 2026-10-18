import { existsSync } from "node:fs";
import path from "node:path";
import { app } from "electron";
import { engineStatus } from "../engine/paths";
import type { EngineInstallEvent } from "../shared/engine";
import type { EnvStatus } from "../shared/ipc";
import type { MainToWorker } from "../shared/worker-protocol";
import { completeLicences } from "./licences";
import { engineDir, onWorkerMessage, spawnWorker, workerEnv } from "./workers";

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
