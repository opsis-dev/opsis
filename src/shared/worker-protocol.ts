import type { EngineInstallEvent, RunRequest } from "./ipc";
import type { RunEvent } from "./schema/events";
import type { AppSettings } from "./settings";

export type WorkerEnv = {
  appRoot: string;
  isPackaged: boolean;
  resourcesPath: string;
  userData: string;
  engineDir: string;
  settings: AppSettings;
};

export type LoginRequest = {
  url: string;
  doneWhen: "url-change" | "selector" | "window-closed";
  selector: string;
  timeoutSec: number;
};

export type MainToWorker =
  | { type: "run"; env: WorkerEnv; request: RunRequest; runId: string }
  | { type: "install-engine"; env: WorkerEnv }
  | { type: "cancel" }
  | { type: "secret:response"; id: string; value?: string; error?: string }
  | { type: "login:response"; id: string; value?: string; error?: string };

export type WorkerToMain =
  | { type: "event"; event: RunEvent }
  | {
      type: "secret:request";
      id: string;
      op: "read" | "write";
      projectDir: string;
      name: string;
      value?: string;
    }
  | { type: "login:request"; id: string; request: LoginRequest }
  | { type: "install:progress"; event: EngineInstallEvent }
  | { type: "install:done"; error?: string };
